use api::{get_current_user, ok_response, unauthorized_response, ApiResponse};
use data_access::plans::get_plans_by_user_id;
use endurance_racing_planner_common::PlanListDto;
use lambda_http::{service_fn, Error, IntoResponse, Request};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let db_context = data_access::initialize().await?;
    let db_context_ref = &db_context;
    let handler = move |event: Request| get_plans(event, db_context_ref);

    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn get_plans(event: Request, db_context: &PgPool) -> Result<impl IntoResponse, Error> {
    let current_user = get_current_user(event.headers(), db_context).await;
    if let Some(user) = current_user {
        let plans = get_plans_by_user_id(db_context, user.id)
            .await?
            .iter()
            .map(|p| PlanListDto {
                id: p.id,
                title: p.title.clone(),
                owner: p.owner.clone(),
                last_modified: p.modified_date.or(Some(p.created_date)).unwrap(),
            })
            .collect::<Vec<PlanListDto>>();

        Ok(ok_response(ApiResponse { body: plans }))
    } else {
        Ok(unauthorized_response())
    }
}
