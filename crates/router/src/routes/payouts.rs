use actix_web::{
    body::{BoxBody, MessageBody},
    web, HttpRequest, HttpResponse, Responder,
};
use router_env::{instrument, tracing, Flow};

use super::app::AppState;
#[cfg(feature = "olap")]
use crate::types::api::payments as payment_types;
use crate::{
    core::{api_locking, payouts::*},
    services::{api, authentication as auth, authorization::permissions::Permission},
    types::api::payouts as payout_types,
};

/// Payouts - Create
#[utoipa::path(
    post,
    path = "/payouts/create",
    request_body=PayoutCreateRequest,
    responses(
        (status = 200, description = "Payout created", body = PayoutCreateResponse),
        (status = 400, description = "Missing Mandatory fields")
    ),
    tag = "Payouts",
    operation_id = "Create a Payout",
    security(("api_key" = []))
)]
#[instrument(skip_all, fields(flow = ?Flow::PayoutsCreate))]
pub async fn payouts_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    json_payload: web::Json<payout_types::PayoutCreateRequest>,
) -> HttpResponse {
    let flow = Flow::PayoutsCreate;
    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        json_payload.into_inner(),
        |state, auth, req, _| {
            payouts_create_core(state, auth.merchant_account, auth.key_store, req)
        },
        &auth::HeaderAuth(auth::ApiKeyAuth),
        api_locking::LockAction::NotApplicable,
    ))
    .await
}
/// Payouts - Retrieve
#[utoipa::path(
    get,
    path = "/payouts/{payout_id}",
    params(
        ("payout_id" = String, Path, description = "The identifier for payout]")
    ),
    responses(
        (status = 200, description = "Payout retrieved", body = PayoutCreateResponse),
        (status = 404, description = "Payout does not exist in our records")
    ),
    tag = "Payouts",
    operation_id = "Retrieve a Payout",
    security(("api_key" = []))
)]
#[instrument(skip_all, fields(flow = ?Flow::PayoutsRetrieve))]
pub async fn payouts_retrieve(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    query_params: web::Query<payout_types::PayoutRetrieveBody>,
) -> HttpResponse {
    let payout_retrieve_request = payout_types::PayoutRetrieveRequest {
        payout_id: path.into_inner(),
        force_sync: query_params.force_sync.to_owned(),
        merchant_id: query_params.merchant_id.to_owned(),
    };
    let flow = Flow::PayoutsRetrieve;
    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        payout_retrieve_request,
        |state, auth, req, _| {
            payouts_retrieve_core(state, auth.merchant_account, None, auth.key_store, req)
        },
        auth::auth_type(
            &auth::HeaderAuth(auth::ApiKeyAuth),
            &auth::JWTAuth(Permission::PayoutRead),
            req.headers(),
        ),
        api_locking::LockAction::NotApplicable,
    ))
    .await
}
/// Payouts - Update
#[utoipa::path(
    post,
    path = "/payouts/{payout_id}",
    params(
        ("payout_id" = String, Path, description = "The identifier for payout]")
    ),
    request_body=PayoutCreateRequest,
    responses(
        (status = 200, description = "Payout updated", body = PayoutCreateResponse),
        (status = 400, description = "Missing Mandatory fields")
    ),
    tag = "Payouts",
    operation_id = "Update a Payout",
    security(("api_key" = []))
)]
#[instrument(skip_all, fields(flow = ?Flow::PayoutsUpdate))]
pub async fn payouts_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    json_payload: web::Json<payout_types::PayoutCreateRequest>,
) -> HttpResponse {
    let flow = Flow::PayoutsUpdate;
    let payout_id = path.into_inner();
    let mut payout_update_payload = json_payload.into_inner();
    payout_update_payload.payout_id = Some(payout_id);
    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        payout_update_payload,
        |state, auth, req, _| {
            payouts_update_core(state, auth.merchant_account, auth.key_store, req)
        },
        &auth::HeaderAuth(auth::ApiKeyAuth),
        api_locking::LockAction::NotApplicable,
    ))
    .await
}

#[instrument(skip_all, fields(flow = ?Flow::PayoutsConfirm))]
pub async fn payouts_confirm(
    state: web::Data<AppState>,
    req: HttpRequest,
    json_payload: web::Json<payout_types::PayoutCreateRequest>,
    path: web::Path<String>,
) -> HttpResponse {
    let flow = Flow::PayoutsConfirm;
    let mut payload = json_payload.into_inner();
    let payout_id = path.into_inner();
    tracing::Span::current().record("payout_id", &payout_id);
    payload.payout_id = Some(payout_id);
    payload.confirm = Some(true);
    let (auth_type, _auth_flow) =
        match auth::check_client_secret_and_get_auth(req.headers(), &payload) {
            Ok(auth) => auth,
            Err(e) => return api::log_and_return_error_response(e),
        };

    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        payload,
        |state, auth, req, _| {
            payouts_confirm_core(state, auth.merchant_account, auth.key_store, req)
        },
        &*auth_type,
        api_locking::LockAction::NotApplicable,
    ))
    .await
}
/// Payouts - Cancel
#[utoipa::path(
    post,
    path = "/payouts/{payout_id}/cancel",
    params(
        ("payout_id" = String, Path, description = "The identifier for payout")
    ),
    request_body=PayoutActionRequest,
    responses(
        (status = 200, description = "Payout cancelled", body = PayoutCreateResponse),
        (status = 400, description = "Missing Mandatory fields")
    ),
    tag = "Payouts",
    operation_id = "Cancel a Payout",
    security(("api_key" = []))
)]
#[instrument(skip_all, fields(flow = ?Flow::PayoutsCancel))]
pub async fn payouts_cancel(
    state: web::Data<AppState>,
    req: HttpRequest,
    json_payload: web::Json<payout_types::PayoutActionRequest>,
    path: web::Path<String>,
) -> HttpResponse {
    let flow = Flow::PayoutsCancel;
    let mut payload = json_payload.into_inner();
    payload.payout_id = path.into_inner();

    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        payload,
        |state, auth, req, _| {
            payouts_cancel_core(state, auth.merchant_account, auth.key_store, req)
        },
        &auth::HeaderAuth(auth::ApiKeyAuth),
        api_locking::LockAction::NotApplicable,
    ))
    .await
}
/// Payouts - Fulfill
#[utoipa::path(
    post,
    path = "/payouts/{payout_id}/fulfill",
    params(
        ("payout_id" = String, Path, description = "The identifier for payout")
    ),
    request_body=PayoutActionRequest,
    responses(
        (status = 200, description = "Payout fulfilled", body = PayoutCreateResponse),
        (status = 400, description = "Missing Mandatory fields")
    ),
    tag = "Payouts",
    operation_id = "Fulfill a Payout",
    security(("api_key" = []))
)]
#[instrument(skip_all, fields(flow = ?Flow::PayoutsFulfill))]
pub async fn payouts_fulfill(
    state: web::Data<AppState>,
    req: HttpRequest,
    json_payload: web::Json<payout_types::PayoutActionRequest>,
    path: web::Path<String>,
) -> HttpResponse {
    let flow = Flow::PayoutsFulfill;
    let mut payload = json_payload.into_inner();
    payload.payout_id = path.into_inner();

    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        payload,
        |state, auth, req, _| {
            payouts_fulfill_core(state, auth.merchant_account, auth.key_store, req)
        },
        &auth::HeaderAuth(auth::ApiKeyAuth),
        api_locking::LockAction::NotApplicable,
    ))
    .await
}

/// Payouts - List
#[cfg(feature = "olap")]
#[utoipa::path(
    get,
    path = "/payouts/list",
    responses(
        (status = 200, description = "Payouts listed", body = PayoutListResponse),
        (status = 404, description = "Payout not found")
    ),
    tag = "Payouts",
    operation_id = "List payouts",
    security(("api_key" = []))
)]
#[instrument(skip_all, fields(flow = ?Flow::PayoutsList))]
pub async fn payouts_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    json_payload: web::Query<payout_types::PayoutListConstraints>,
) -> HttpResponse {
    let flow = Flow::PayoutsList;
    let payload = json_payload.into_inner();

    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        payload,
        |state, auth, req, _| {
            payouts_list_core(state, auth.merchant_account, None, auth.key_store, req)
        },
        auth::auth_type(
            &auth::HeaderAuth(auth::ApiKeyAuth),
            &auth::JWTAuth(Permission::PayoutRead),
            req.headers(),
        ),
        api_locking::LockAction::NotApplicable,
    ))
    .await
}

/// Payouts - Filtered list
#[cfg(feature = "olap")]
#[utoipa::path(
    post,
    path = "/payouts/list",
    responses(
        (status = 200, description = "Payouts filtered", body = PayoutListResponse),
        (status = 404, description = "Payout not found")
    ),
    tag = "Payouts",
    operation_id = "Filter payouts",
    security(("api_key" = []))
)]
#[instrument(skip_all, fields(flow = ?Flow::PayoutsList))]
pub async fn payouts_list_by_filter(
    state: web::Data<AppState>,
    req: HttpRequest,
    json_payload: web::Json<payout_types::PayoutListFilterConstraints>,
) -> HttpResponse {
    let flow = Flow::PayoutsList;
    let payload = json_payload.into_inner();

    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        payload,
        |state, auth, req, _| {
            payouts_filtered_list_core(state, auth.merchant_account, None, auth.key_store, req)
        },
        auth::auth_type(
            &auth::HeaderAuth(auth::ApiKeyAuth),
            &auth::JWTAuth(Permission::PayoutRead),
            req.headers(),
        ),
        api_locking::LockAction::NotApplicable,
    ))
    .await
}

/// Payouts - Available filters
#[cfg(feature = "olap")]
#[utoipa::path(
    post,
    path = "/payouts/filter",
    responses(
        (status = 200, description = "Payouts filtered", body = PayoutListFilters),
        (status = 404, description = "Payout not found")
    ),
    tag = "Payouts",
    operation_id = "Filter payouts",
    security(("api_key" = []))
)]
#[instrument(skip_all, fields(flow = ?Flow::PayoutsFilter))]
pub async fn payouts_list_available_filters(
    state: web::Data<AppState>,
    req: HttpRequest,
    json_payload: web::Json<payment_types::TimeRange>,
) -> HttpResponse {
    let flow = Flow::PayoutsFilter;
    let payload = json_payload.into_inner();

    Box::pin(api::server_wrap(
        flow,
        state,
        &req,
        payload,
        |state, auth, req, _| {
            payouts_list_available_filters_core(state, auth.merchant_account, req)
        },
        auth::auth_type(
            &auth::HeaderAuth(auth::ApiKeyAuth),
            &auth::JWTAuth(Permission::PayoutRead),
            req.headers(),
        ),
        api_locking::LockAction::NotApplicable,
    ))
    .await
}

#[instrument(skip_all, fields(flow = ?Flow::PayoutsAccounts))]
// #[get("/accounts")]
pub async fn payouts_accounts() -> impl Responder {
    let _flow = Flow::PayoutsAccounts;
    http_response("accounts")
}

fn http_response<T: MessageBody + 'static>(response: T) -> HttpResponse<BoxBody> {
    HttpResponse::Ok().body(response)
}
