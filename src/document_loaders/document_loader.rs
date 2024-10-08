use std::pin::Pin;

use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use futures_util::{pin_mut, StreamExt};

use crate::{schemas::Document, text_splitter::TextSplitter};

use super::LoaderError;

#[async_trait]
pub trait Loader<F, M, T>: Send + Sync {
    async fn load(
        self,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<Document, LoaderError>> + Send + 'static>>,
        LoaderError,
    >
    where
        F: Fn(&T) -> bool + Send + Sync,
        M: Fn(&T) -> Result<Document, LoaderError> + Send + Sync,
        T: Send + Sync;

    async fn load_and_split<TS: TextSplitter + 'static>(
        self,
        splitter: TS,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<Document, LoaderError>> + Send + 'static>>,
        LoaderError,
    >;
}

pub(crate) async fn process_doc_stream<TS: TextSplitter + 'static>(
    doc_stream: Pin<Box<dyn Stream<Item = Result<Document, LoaderError>> + Send>>,
    splitter: TS,
) -> impl Stream<Item = Result<Document, LoaderError>> {
    stream! {
        pin_mut!(doc_stream);
        while let Some(doc_result) = doc_stream.next().await {
            match doc_result {
                Ok(doc) => {
                    match splitter.split_documents(&[doc]).await {
                        Ok(docs) => {
                            for doc in docs {
                                yield Ok(doc);
                            }
                        },
                        Err(e) => yield Err(LoaderError::TextSplitterError(e)),
                    }
                }
                Err(e) => yield Err(e),
            }
        }
    }
}
