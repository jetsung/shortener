mod history_service;
mod shorten_service;

pub use history_service::{HistoryResponse, HistoryService, UserAgentInfo};
pub use shorten_service::{
    CreateShortenRequest, PageMeta, PagedResponse, ShortenResponse, ShortenService,
    UpdateShortenRequest,
};
