use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;
use std::sync::Arc;

use crate::modules::nft::manager::NFTManager;
use crate::modules::nft::models::*;

pub struct AppState {
    pub nft_manager: Arc<NFTManager>,
}

pub async fn create_nft(
    data: web::Data<AppState>,
    request: web::Json<CreateNFTRequest>,
) -> ActixResult<HttpResponse> {
    match data.nft_manager.create_nft(request.into_inner()).await {
        Ok(nft_info) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": nft_info
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn mint_nft(
    data: web::Data<AppState>,
    request: web::Json<MintNFTRequest>,
) -> ActixResult<HttpResponse> {
    match data.nft_manager.mint_nft(request.into_inner()).await {
        Ok(signature) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "signature": signature.to_string()
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn transfer_nft(
    data: web::Data<AppState>,
    request: web::Json<TransferNFTRequest>,
) -> ActixResult<HttpResponse> {
    match data.nft_manager.transfer_nft(request.into_inner()).await {
        Ok(signature) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "signature": signature.to_string()
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn burn_nft(
    data: web::Data<AppState>,
    request: web::Json<BurnNFTRequest>,
) -> ActixResult<HttpResponse> {
    match data.nft_manager.burn_nft(request.into_inner()).await {
        Ok(signature) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "signature": signature.to_string()
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn get_nft_info(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    match data.nft_manager.get_nft_info(&path.into_inner()).await {
        Ok(nft_info) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": nft_info
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn get_nft_owner(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let mint = path.into_inner();
    match data.nft_manager.get_nft_owner(&mint).await {
        Ok(owner) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "mint": mint,
                "owner": owner
            }
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}
