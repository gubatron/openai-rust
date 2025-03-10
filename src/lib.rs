#![doc = include_str!("../README.md")]
//#![feature(str_split_remainder)]
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use reqwest;

pub extern crate futures_util;

lazy_static! {
    static ref DEFAULT_BASE_URL: reqwest::Url =
        reqwest::Url::parse("https://api.openai.com/v1/models").unwrap();
}

/// This is the main interface to interact with the api.
pub struct Client {
    req_client: reqwest::Client,
    key: String,
    base_url: reqwest::Url,
}

pub mod chat;
pub mod completions;
pub mod edits;
pub mod embeddings;
pub mod images;
pub mod models;

impl Client {
    /// Create a new client.
    /// This will automatically build a [reqwest::Client] used internally.
    pub fn new(api_key: &str) -> Client {
        let req_client = reqwest::ClientBuilder::new().build().unwrap();
        Client {
            req_client,
            key: api_key.to_owned(),
            base_url: DEFAULT_BASE_URL.clone(),
        }
    }

    /// Build a client using your own [reqwest::Client].
    pub fn new_with_client(api_key: &str, req_client: reqwest::Client) -> Client {
        Client {
            req_client,
            key: api_key.to_owned(),
            base_url: DEFAULT_BASE_URL.clone(),
        }
    }

    // Build a client with a custom base url. The default is `https://api.openai.com/v1/models`
    pub fn new_with_base_url(api_key: &str, base_url: &str) -> Client {
        let req_client = reqwest::ClientBuilder::new().build().unwrap();
        let base_url = reqwest::Url::parse(base_url).unwrap();
        Client {
            req_client,
            key: api_key.to_owned(),
            base_url,
        }
    }

    pub fn new_with_client_and_base_url(
        api_key: &str,
        req_client: reqwest::Client,
        base_url: &str,
    ) -> Client {
        Client {
            req_client,
            key: api_key.to_owned(),
            base_url: reqwest::Url::parse(base_url).unwrap(),
        }
    }

    /// List and describe the various models available in the API. You can refer to the [Models](https://platform.openai.com/docs/models) documentation to understand what models are available and the differences between them.
    ///
    /// ```
    /// # use openai_rust2 as openai_rust;
    /// # let api_key = "";
    /// # tokio_test::block_on(async {
    /// let client = openai_rust::Client::new(api_key);
    /// let models = client.list_models(None).await.unwrap();
    /// # })
    /// ```
    ///
    /// See <https://platform.openai.com/docs/api-reference/models/list>.
    pub async fn list_models(
        &self,
        opt_url_path: Option<String>,
    ) -> Result<Vec<models::Model>, anyhow::Error> {
        let mut url = self.base_url.clone();
        url.set_path(&opt_url_path.unwrap_or_else(|| String::from("/v1/models")));

        let res = self
            .req_client
            .get(url)
            .bearer_auth(&self.key)
            .send()
            .await?;

        if res.status() == 200 {
            Ok(res.json::<models::ListModelsResponse>().await?.data)
        } else {
            Err(anyhow!(res.text().await?))
        }
    }

    /// Given a list of messages comprising a conversation, the model will return a response.
    ///
    /// See <https://platform.openai.com/docs/api-reference/chat>.
    /// ```
    /// # use tokio_test;
    /// # tokio_test::block_on(async {
    /// # use openai_rust2 as openai_rust;
    /// # let api_key = "";
    /// let client = openai_rust::Client::new(api_key);
    /// let args = openai_rust::chat::ChatArguments::new("gpt-3.5-turbo", vec![
    ///    openai_rust::chat::Message {
    ///        role: "user".to_owned(),
    ///        content: "Hello GPT!".to_owned(),
    ///    }
    /// ]);
    /// let res = client.create_chat(args, None).await.unwrap();
    /// println!("{}", res.choices[0].message.content);
    /// # })
    /// ```
    pub async fn create_chat(
        &self,
        args: chat::ChatArguments,
        opt_url_path: Option<String>,
    ) -> Result<chat::ChatCompletion, anyhow::Error> {
        let mut url = self.base_url.clone();
        url.set_path(&opt_url_path.unwrap_or_else(|| String::from("/v1/chat/completions")));

        let res = self
            .req_client
            .post(url)
            .bearer_auth(&self.key)
            .json(&args)
            .send()
            .await?;

        if res.status() == 200 {
            Ok(res.json().await?)
        } else {
            Err(anyhow!(res.text().await?))
        }
    }

    /// Like [Client::create_chat] but with streaming.
    ///
    /// See <https://platform.openai.com/docs/api-reference/chat>.
    ///
    /// This method will return a stream of [chat::stream::ChatCompletionChunk]s. Use with [futures_util::StreamExt::next].
    ///
    /// ```
    /// # use tokio_test;
    /// # tokio_test::block_on(async {
    /// # use openai_rust2 as openai_rust;
    /// # use std::io::Write;
    /// # let client = openai_rust::Client::new("");
    /// # let args = openai_rust::chat::ChatArguments::new("gpt-3.5-turbo", vec![
    /// #    openai_rust::chat::Message {
    /// #        role: "user".to_owned(),
    /// #        content: "Hello GPT!".to_owned(),
    /// #    }
    /// # ]);
    /// use openai_rust::futures_util::StreamExt;
    /// let mut res = client.create_chat_stream(args, None).await.unwrap();
    /// while let Some(chunk) = res.next().await {
    ///     print!("{}", chunk.unwrap());
    ///     std::io::stdout().flush().unwrap();
    /// }
    /// # })
    /// ```
    ///
    pub async fn create_chat_stream(
        &self,
        args: chat::ChatArguments,
        opt_url_path: Option<String>,
    ) -> Result<chat::stream::ChatCompletionChunkStream> {
        let mut url = self.base_url.clone();
        url.set_path(&opt_url_path.unwrap_or_else(|| String::from("/v1/chat/completions")));

        // Enable streaming
        let mut args = args;
        args.stream = Some(true);

        let res = self
            .req_client
            .post(url)
            .bearer_auth(&self.key)
            .json(&args)
            .send()
            .await?;

        if res.status() == 200 {
            Ok(chat::stream::ChatCompletionChunkStream::new(Box::pin(
                res.bytes_stream(),
            )))
        } else {
            Err(anyhow!(res.text().await?))
        }
    }

    /// Given a prompt, the model will return one or more predicted completions, and can also return the probabilities of alternative tokens at each position.
    ///
    /// See <https://platform.openai.com/docs/api-reference/completions>
    ///
    /// ```
    /// # use openai_rust2 as openai_rust;
    /// # use openai_rust::*;
    /// # use tokio_test;
    /// # tokio_test::block_on(async {
    /// # let api_key = "";
    /// let c = Client::new(api_key);
    /// let args = completions::CompletionArguments::new("text-davinci-003", "The quick brown fox".to_owned());
    /// println!("{}", c.create_completion(args, None).await.unwrap().choices[0].text);
    /// # })
    /// ```
    pub async fn create_completion(
        &self,
        args: completions::CompletionArguments,
        opt_url_path: Option<String>,
    ) -> Result<completions::CompletionResponse> {
        let mut url = self.base_url.clone();
        url.set_path(&opt_url_path.unwrap_or_else(|| String::from("/v1/completions")));

        let res = self
            .req_client
            .post(url)
            .bearer_auth(&self.key)
            .json(&args)
            .send()
            .await?;

        if res.status() == 200 {
            Ok(res.json().await?)
        } else {
            Err(anyhow!(res.text().await?))
        }
    }

    /// Get a vector representation of a given input that can be easily consumed by machine learning models and algorithms.
    ///
    /// See <https://platform.openai.com/docs/api-reference/embeddings>
    ///
    /// ```
    /// # use openai_rust2 as openai_rust;
    /// # use tokio_test;
    /// # tokio_test::block_on(async {
    /// # let api_key = "";
    /// let c = openai_rust::Client::new(api_key);
    /// let args = openai_rust::embeddings::EmbeddingsArguments::new("text-embedding-ada-002", "The food was delicious and the waiter...".to_owned());
    /// println!("{:?}", c.create_embeddings(args, None).await.unwrap().data);
    /// # })
    /// ```
    ///
    pub async fn create_embeddings(
        &self,
        args: embeddings::EmbeddingsArguments,
        opt_url_path: Option<String>,
    ) -> Result<embeddings::EmbeddingsResponse> {
        let mut url = self.base_url.clone();
        url.set_path(&opt_url_path.unwrap_or_else(|| String::from("/v1/embeddings")));

        let res = self
            .req_client
            .post(url)
            .bearer_auth(&self.key)
            .json(&args)
            .send()
            .await?;

        if res.status() == 200 {
            Ok(res.json().await?)
        } else {
            Err(anyhow!(res.text().await?))
        }
    }

    /// Creates an image given a prompt.
    pub async fn create_image(
        &self,
        args: images::ImageArguments,
        opt_url_path: Option<String>,
    ) -> Result<Vec<String>> {
        let mut url = self.base_url.clone();
        url.set_path(&opt_url_path.unwrap_or_else(|| String::from("/v1/images/generations")));

        let res = self
            .req_client
            .post(url)
            .bearer_auth(&self.key)
            .json(&args)
            .send()
            .await?;

        if res.status() == 200 {
            Ok(res
                .json::<images::ImageResponse>()
                .await?
                .data
                .iter()
                .map(|o| match o {
                    images::ImageObject::Url(s) => s.to_string(),
                    images::ImageObject::Base64JSON(s) => s.to_string(),
                })
                .collect())
        } else {
            Err(anyhow!(res.text().await?))
        }
    }
}
