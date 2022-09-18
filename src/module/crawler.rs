use async_trait::async_trait;
use crate::module::error::Error;
use reqwest::{ Client, header };
use regex::Regex;
use serde::{ Deserialize, Serialize };
use select::{ document::Document, predicate::{ Attr, Class, Name, Predicate } };
use tokio::sync::Mutex;
use fantoccini::{ Client as HttpClient, ClientBuilder };
use std::{
    collections::HashSet,
    sync::{
        atomic::{ AtomicUsize, Ordering },
        Arc,
    },
};
use std::time::Duration;
use tokio::{
    sync::{ mpsc, Barrier },
    time::sleep,
};
use futures::stream::StreamExt;

#[async_trait]
pub trait Spider: Send + Sync {
    type Item;
    fn name(&self) -> String;
    fn start_urls(&self) -> Vec<String>;
    async fn scrapy(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error>;
    async fn process(&self, item: Self::Item) -> Result<(), Error>;
}

pub struct CveDetails {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct Cve {
    name: String,
    url: String,
    cwe_id: Option<String>,
    cwe_url: Option<String>,
    vulnerability_type: String,
    publish_date: String,
    update_date: String,
    score: f32,
    access: String,
    complexity: String,
    authentication: String,
    confidentiality: String,
    integrity: String,
    availability: String,
}

impl CveDetails {
    pub fn new() -> Self {
        let timeout = Duration::from_secs(6);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("spiders/cvedetails: Building HTTP client");

        CveDetails { client }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubItem {
    login: String,
    id: u64,
    node_id: String,
    html_url: String,
    avatar_url: String,
}

pub struct GitHubSpider {
    client: Client,
    page_regex: Regex,
    expected_number_of_results: usize,
}

#[derive(Debug, Clone)]
pub struct QuotesItem {
    quote: String,
    author: String,
}

pub struct QuotesSpider {
    webdriver_client: Mutex<HttpClient>,
}

#[async_trait]
impl Spider for CveDetails {
    type Item = Cve;

    fn name(&self) -> String {
        String::from("cvedetails")
    }

    fn start_urls(&self) -> Vec<String> {
        vec!["https://www.cvedetails.com/vulnerability-list/vulnerabilities.html".to_string()]
    }

    async fn scrapy(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        log::info!("visiting: {}", url);

        let http_res = self.client.get(url).send().await?.text().await?;
        let mut items = Vec::new();

        let document = Document::from(http_res.as_str());

        let rows = document.select(Attr("id", "vulnslisttable").descendant(Class("srrowns")));
        for row in rows {
            let mut columns = row.select(Name("td"));
            let _ = columns.next();
            let cve_link = columns.next().unwrap().select(Name("a")).next().unwrap();
            let cve_name = cve_link.text().trim().to_string();
            let cve_url = self.url_join(cve_link.attr("href").unwrap());

            let cwe = columns
                .next()
                .unwrap()
                .select(Name("a"))
                .next()
                .map(|cwe_link| {
                    (
                        cwe_link.text().trim().to_string(),
                        self.url_join(cwe_link.attr("href").unwrap()),
                    )
                });

            let _ = columns.next();

            let vulnerability_type = columns.next().unwrap().text().trim().to_string();

            let publish_date = columns.next().unwrap().text().trim().to_string();
            let update_date = columns.next().unwrap().text().trim().to_string();

            let score: f32 = columns
                .next()
                .unwrap()
                .text()
                .trim()
                .to_string()
                .parse()
                .unwrap();

            let _ = columns.next();

            let access = columns.next().unwrap().text().trim().to_string();
            let complexity = columns.next().unwrap().text().trim().to_string();
            let authentication = columns.next().unwrap().text().trim().to_string();
            let confidentiality = columns.next().unwrap().text().trim().to_string();
            let integrity = columns.next().unwrap().text().trim().to_string();
            let availability = columns.next().unwrap().text().trim().to_string();

            let cve = Cve {
                name: cve_name,
                url: cve_url,
                cwe_id: cwe.as_ref().map(|cwe| cwe.0.clone()),
                cwe_url: cwe.as_ref().map(|cwe| cwe.1.clone()),
                vulnerability_type,
                publish_date,
                update_date,
                score,
                access,
                complexity,
                authentication,
                confidentiality,
                integrity,
                availability,
            };
            items.push(cve);
        }

        let next_pages_links = document
            .select(Attr("id", "pagingb").descendant(Name("a")))
            .filter_map(|n| n.attr("href"))
            .map(|url| self.url_join(url))
            .collect::<Vec<String>>();

        Ok((items, next_pages_links))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        println!("{:?}", item);

        Ok(())
    }
}

impl CveDetails {
    fn url_join(&self, url: &str) -> String {
        let url = url.trim();

        if url.starts_with("//www.cvedetails.com") {
            return format!("https:{}", url);
        } else if url.starts_with("/") {
            return format!("https://www.cvedetails.com{}", url);
        }

        return url.to_string();
    }
}

impl GitHubSpider {
    pub fn new() -> Self {
        let timeout = Duration::from_secs(6);
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        let client = Client::builder()
            .timeout(timeout)
            .default_headers(headers)
            .user_agent(
                "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:47.0) Gecko/20100101 Firefox/47.0",
            )
            .build()
            .expect("spiders/github: Building HTTP client");

        let page_regex =
            Regex::new(".*page=([0-9]*).*").expect("spiders/github: Compiling page regex");

        GitHubSpider {
            client,
            page_regex,
            expected_number_of_results: 100,
        }
    }
}


#[async_trait]
impl Spider for GitHubSpider {
    type Item = GitHubItem;

    fn name(&self) -> String {
        String::from("github")
    }

    fn start_urls(&self) -> Vec<String> {
        vec!["https://api.github.com/orgs/google/public_members?per_page=100&page=1".to_string()]
    }

    async fn scrapy(&self, url: String) -> Result<(Vec<GitHubItem>, Vec<String>), Error> {
        let items: Vec<GitHubItem> = self.client.get(&url).send().await?.json().await?;

        let next_pages_links = if items.len() == self.expected_number_of_results {
            let captures = self.page_regex.captures(&url).unwrap();
            let old_page_number = captures.get(1).unwrap().as_str().to_string();
            let mut new_page_number = old_page_number
                .parse::<usize>()
                .map_err(|_| Error::Internal("spider/github: parsing page number".to_string()))?;
            new_page_number += 1;

            let next_url = url.replace(
                format!("&page={}", old_page_number).as_str(),
                format!("&page={}", new_page_number).as_str(),
            );
            vec![next_url]
        } else {
            Vec::new()
        };

        Ok((items, next_pages_links))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        println!("{}, {}, {}", item.login, item.html_url, item.avatar_url);

        Ok(())
    }
}

impl QuotesSpider {
    pub async fn new() -> Result<Self, Error> {
        let mut caps = serde_json::map::Map::new();
        let chrome_opts = serde_json::json!({ "args": ["--headless", "--disable-gpu"] });
        caps.insert("goog:chromeOptions".to_string(), chrome_opts);
        let webdriver_client = ClientBuilder::rustls()
            .capabilities(caps)
            .connect("http://localhost:4444")
            .await?;

        Ok(QuotesSpider {
            webdriver_client: Mutex::new(webdriver_client),
        })
    }
}


#[async_trait]
impl Spider for QuotesSpider {
    type Item = QuotesItem;

    fn name(&self) -> String {
        String::from("quotes")
    }

    fn start_urls(&self) -> Vec<String> {
        vec!["https://quotes.toscrape.com/js".to_string()]
    }

    async fn scrapy(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        let mut items = Vec::new();
        let html = {
            let webdriver = self.webdriver_client.lock().await;
            webdriver.goto(&url).await?;
            webdriver.source().await?
        };

        let document = Document::from(html.as_str());

        let quotes = document.select(Class("quote"));
        for quote in quotes {
            let mut spans = quote.select(Name("span"));
            let quote_span = spans.next().unwrap();
            let quote_str = quote_span.text().trim().to_string();

            let author = spans
                .next()
                .unwrap()
                .select(Class("author"))
                .next()
                .unwrap()
                .text()
                .trim()
                .to_string();

            items.push(QuotesItem {
                quote: quote_str,
                author,
            });
        }

        let next_pages_link = document
            .select(
                Class("pager")
                    .descendant(Class("next"))
                    .descendant(Name("a")),
            )
            .filter_map(|n| n.attr("href"))
            .map(|url| self.url_join(url))
            .collect::<Vec<String>>();

        Ok((items, next_pages_link))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        println!("{}", item.quote);
        println!("by {}\n", item.author);
        Ok(())
    }
}

impl QuotesSpider {
    fn url_join(&self, url: &str) -> String {
        let url = url.trim();

        if url.starts_with("/") {
            return format!("https://quotes.toscrape.com{}", url);
        }

        return url.to_string();
    }
}

pub struct Crawler {
    delay: Duration,
    concurrency_count: usize,
    processing_count: usize,
}

impl Crawler {
    pub fn new(
        delay: Duration,
        concurrency_count: usize,
        processing_count: usize,
    ) -> Self {
        Crawler {
            delay,
            concurrency_count,
            processing_count,
        }
    }

    pub async fn run<T: Send + 'static>(&self, spider: Arc<dyn Spider<Item = T>>) {
        let mut visited_urls = HashSet::<String>::new();
        let concurrency_count = self.concurrency_count;
        let concurrency_queue_capacity = concurrency_count * 400;
        let processing_count = self.processing_count;
        let processing_queue_capacity = processing_count * 10;
        let active_spiders = Arc::new(AtomicUsize::new(0));

        let (urls_to_visit_tx, urls_to_visit_rx) = mpsc::channel(concurrency_queue_capacity);
        let (items_tx, items_rx) = mpsc::channel(processing_queue_capacity);
        let (new_urls_tx, mut new_urls_rx) = mpsc::channel(concurrency_queue_capacity);
        let barrier = Arc::new(Barrier::new(3));

        for url in spider.start_urls() {
            visited_urls.insert(url.clone());
            let _ = urls_to_visit_tx.send(url).await;
        }

        self.processors(
            processing_count,
            spider.clone(),
            items_rx,
            barrier.clone(),
        );

        self.scrapers(
            concurrency_count,
            spider.clone(),
            urls_to_visit_rx,
            new_urls_tx.clone(),
            items_tx,
            active_spiders.clone(),
            self.delay,
            barrier.clone(),
        );

        loop {
            if let Some((visited_url, new_urls)) = new_urls_rx.try_recv().ok() {
                visited_urls.insert(visited_url);

                for url in new_urls {
                    if !visited_urls.contains(&url) {
                        visited_urls.insert(url.clone());
                        log::debug!("queueing: {}", url);
                        let _ = urls_to_visit_tx.send(url).await;
                    }
                }
            }

            if new_urls_tx.capacity() == concurrency_queue_capacity
            && urls_to_visit_tx.capacity() == concurrency_queue_capacity
            && active_spiders.load(Ordering::SeqCst) == 0
            {
                break;
            }

            sleep(Duration::from_millis(5)).await;
        }

        log::info!("crawler: control loop exited");

        drop(urls_to_visit_tx);

        barrier.wait().await;
    }

    fn processors<T: Send + 'static>(
        &self,
        concurrency: usize,
        spider: Arc<dyn Spider<Item = T>>,
        items: mpsc::Receiver<T>,
        barrier: Arc<Barrier>,
    ) {
        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(items)
                .for_each_concurrent(concurrency, |item| async {
                    let _ = spider.process(item).await;
                })
                .await;

            barrier.wait().await;
        });
    }

    fn scrapers<T: Send + 'static>(
        &self,
        concurrency: usize,
        spider: Arc<dyn Spider<Item = T>>,
        urls_to_vist: mpsc::Receiver<String>,
        new_urls: mpsc::Sender<(String, Vec<String>)>,
        items_tx: mpsc::Sender<T>,
        active_spiders: Arc<AtomicUsize>,
        delay: Duration,
        barrier: Arc<Barrier>,
    ) {
        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(urls_to_vist)
                .for_each_concurrent(concurrency, |queued_url| {
                    let queued_url = queued_url.clone();
                    async {
                        active_spiders.fetch_add(1, Ordering::SeqCst);
                        let mut urls = Vec::new();
                        let res = spider
                            .scrapy(queued_url.clone())
                            .await
                            .map_err(|err| {
                                log::error!("{}", err);
                                err
                            })
                            .ok();

                        if let Some((items, new_urls)) = res {
                            for item in items {
                                let _ = items_tx.send(item).await;
                            }
                            urls = new_urls;
                        }

                        let _ = new_urls.send((queued_url, urls)).await;
                        sleep(delay).await;
                        active_spiders.fetch_sub(1, Ordering::SeqCst);
                    }
                })
                .await;

            drop(items_tx);
            barrier.wait().await;
        });
    }
}

