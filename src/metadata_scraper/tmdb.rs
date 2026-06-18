//! Thin async client for the TMDB v3 API. The scraper builds on this to
//! turn a library file name into rich metadata (poster, plot, cast,
//! collection) by title-match + year-match against TMDB search.

use serde::Deserialize;

const API_BASE: &str = "https://api.themoviedb.org/3";
pub const IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

#[derive(Debug, Clone)]
pub struct TmdbClient {
    api_key: String,
    language: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    id: i64,
    title: String,
    #[serde(default, rename = "original_title")]
    original_title: Option<String>,
    #[serde(default, rename = "release_date")]
    release_date: Option<String>,
    #[serde(default, rename = "overview")]
    overview: Option<String>,
    #[serde(default, rename = "vote_average")]
    vote_average: Option<f64>,
    #[serde(default, rename = "genre_ids")]
    genre_ids: Vec<i64>,
    #[serde(default, rename = "popularity")]
    popularity: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
struct Genre {
    id: i64,
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectionRef {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct GenreList {
    genres: Vec<Genre>,
}

#[derive(Debug, Deserialize)]
struct MovieDetail {
    id: i64,
    title: String,
    #[serde(default, rename = "original_title")]
    original_title: Option<String>,
    #[serde(default, rename = "release_date")]
    release_date: Option<String>,
    #[serde(default, rename = "runtime")]
    runtime: Option<i32>,
    #[serde(default, rename = "overview")]
    overview: Option<String>,
    #[serde(default, rename = "vote_average")]
    vote_average: Option<f64>,
    #[serde(default, rename = "poster_path")]
    poster_path: Option<String>,
    #[serde(default, rename = "backdrop_path")]
    backdrop_path: Option<String>,
    #[serde(default, rename = "belongs_to_collection")]
    belongs_to_collection: Option<CollectionRef>,
    #[serde(default, rename = "genres")]
    genres: Vec<Genre>,
    #[serde(default, rename = "credits")]
    credits: Option<Credits>,
}

#[derive(Debug, Deserialize)]
struct Credits {
    #[serde(default, rename = "cast")]
    cast: Vec<CastMember>,
    #[serde(default, rename = "crew")]
    crew: Vec<CrewMember>,
}

#[derive(Debug, Deserialize)]
struct CastMember {
    name: String,
    #[serde(default, rename = "order")]
    _order: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CrewMember {
    name: String,
    #[serde(default, rename = "job")]
    job: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectionDetail {
    pub id: i64,
    pub name: String,
    #[serde(default, rename = "overview")]
    pub overview: Option<String>,
    #[serde(default, rename = "poster_path")]
    pub poster_path: Option<String>,
    #[serde(default, rename = "backdrop_path")]
    pub backdrop_path: Option<String>,
    #[serde(default, rename = "parts")]
    pub parts: Vec<CollectionPart>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectionPart {
    pub id: i64,
    #[serde(default, rename = "release_date")]
    release_date: Option<String>,
}

/// What the scraper consumes from TMDB. One MovieDetail, normalised.
#[derive(Debug, Clone)]
pub struct TmdbMovie {
    pub tmdb_id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<i32>,
    pub plot: Option<String>,
    pub rating: Option<f64>,
    pub genres: Vec<String>,
    pub director: Option<String>,
    pub cast: Vec<String>,
    pub runtime_minutes: Option<i32>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub collection: Option<CollectionRef>,
}

impl TmdbClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            language: "zh-CN".to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn search(&self, title: &str, year: Option<i32>) -> Result<Option<TmdbMovie>, String> {
        let genres = self.genres().await.unwrap_or_default();

        let mut url = format!(
            "{}/search/movie?api_key={}&query={}&language={}&include_adult=false",
            API_BASE,
            self.api_key,
            urlencoded(title),
            self.language,
        );
        if let Some(y) = year {
            url.push_str(&format!("&year={}", y));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("TMDB search send: {}", e))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("TMDB search body: {}", e))?;
        if !status.is_success() {
            return Err(format!("TMDB search HTTP {}: {}", status, first_line(&text)));
        }
        let parsed: SearchResponse = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => return Err(format!("TMDB search parse: {}", e)),
        };
        let hit = parsed.results.into_iter().max_by(|a, b| {
            let pa = a.popularity.unwrap_or(0.0);
            let pb = b.popularity.unwrap_or(0.0);
            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(hit.map(|h| self.search_hit_to_movie(h, &genres)))
    }

    pub async fn details(&self, tmdb_id: i64) -> Result<Option<TmdbMovie>, String> {
        let url = format!(
            "{}/movie/{}?api_key={}&language={}&append_to_response=credits",
            API_BASE, tmdb_id, self.api_key, self.language,
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("TMDB details send: {}", e))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("TMDB details body: {}", e))?;
        if !status.is_success() {
            return Err(format!("TMDB details HTTP {}: {}", status, first_line(&text)));
        }
        let parsed: MovieDetail = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => return Err(format!("TMDB details parse: {}", e)),
        };
        let director = parsed
            .credits
            .as_ref()
            .and_then(|c| {
                c.crew
                    .iter()
                    .find(|m| m.job == "Director")
                    .map(|m| m.name.clone())
            });
        let cast = parsed
            .credits
            .as_ref()
            .map(|c| c.cast.iter().take(8).map(|m| m.name.clone()).collect())
            .unwrap_or_default();
        let year = parsed
            .release_date
            .as_deref()
            .and_then(|d| d.split('-').next())
            .and_then(|y| y.parse::<i32>().ok());
        Ok(Some(TmdbMovie {
            tmdb_id: parsed.id,
            title: parsed.title,
            original_title: parsed.original_title,
            year,
            plot: parsed.overview,
            rating: parsed.vote_average,
            genres: parsed.genres.into_iter().map(|g| g.name).collect(),
            director,
            cast,
            runtime_minutes: parsed.runtime,
            poster_path: parsed.poster_path,
            backdrop_path: parsed.backdrop_path,
            collection: parsed.belongs_to_collection,
        }))
    }

    pub async fn collection(
        &self,
        id: i64,
    ) -> Result<Option<CollectionDetail>, String> {
        let url = format!(
            "{}/collection/{}?api_key={}&language={}",
            API_BASE, id, self.api_key, self.language,
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("TMDB collection send: {}", e))?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        let text = resp
            .text()
            .await
            .map_err(|e| format!("TMDB collection body: {}", e))?;
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("TMDB collection parse: {}", e))
    }

    async fn genres(&self) -> Result<Vec<Genre>, String> {
        let url = format!(
            "{}/genre/movie/list?api_key={}&language={}",
            API_BASE, self.api_key, self.language
        );
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("TMDB genres send: {}", e))?;
        if !resp.status().is_success() {
            return Ok(Vec::new());
        }
        let text = resp
            .text()
            .await
            .map_err(|e| format!("TMDB genres body: {}", e))?;
        let parsed: GenreList = serde_json::from_str(&text).unwrap_or(GenreList { genres: vec![] });
        Ok(parsed.genres)
    }

    fn search_hit_to_movie(&self, hit: SearchHit, genres: &[Genre]) -> TmdbMovie {
        let year = hit
            .release_date
            .as_deref()
            .and_then(|d| d.split('-').next())
            .and_then(|y| y.parse::<i32>().ok());
        let names: Vec<String> = hit
            .genre_ids
            .iter()
            .filter_map(|id| genres.iter().find(|g| g.id == *id).map(|g| g.name.clone()))
            .collect();
        TmdbMovie {
            tmdb_id: hit.id,
            title: hit.title,
            original_title: hit.original_title,
            year,
            plot: hit.overview,
            rating: hit.vote_average,
            genres: names,
            director: None,
            cast: Vec::new(),
            runtime_minutes: None,
            poster_path: None,
            backdrop_path: None,
            collection: None,
        }
    }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or("")
}

fn urlencoded(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

