use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;
use std::sync::Arc;

use crate::modules::token::manager::TokenManager;
use crate::modules::token::models::*;

pub struct AppState {
    pub token_manager: Arc<TokenManager>,
}

pub async fn create_token(
    data: web::Data<AppState>,
    request: web::Json<CreateTokenRequest>,
) -> ActixResult<HttpResponse> {
    match data.token_manager.create_token(request.into_inner()).await {
        Ok(token_info) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": token_info
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn mint_tokens(
    data: web::Data<AppState>,
    request: web::Json<MintRequest>,
) -> ActixResult<HttpResponse> {
    match data.token_manager.mint_tokens(request.into_inner()).await {
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

pub async fn burn_tokens(
    data: web::Data<AppState>,
    request: web::Json<BurnRequest>,
) -> ActixResult<HttpResponse> {
    match data.token_manager.burn_tokens(request.into_inner()).await {
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

pub async fn transfer_tokens(
    data: web::Data<AppState>,
    request: web::Json<TransferRequest>,
) -> ActixResult<HttpResponse> {
    match data
        .token_manager
        .transfer_tokens(request.into_inner())
        .await
    {
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

pub async fn get_token_info(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    match data.token_manager.get_token_info(&path.into_inner()).await {
        Ok(token_info) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": token_info
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn get_token_balance(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> ActixResult<HttpResponse> {
    let (mint, owner) = path.into_inner();
    match data.token_manager.get_token_balance(&mint, &owner).await {
        Ok(balance) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "mint": mint,
                "owner": owner,
                "balance": balance
            }
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}
