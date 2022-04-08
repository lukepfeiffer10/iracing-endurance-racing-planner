use lambda_http::{service_fn, Error, IntoResponse, Request, Response, Body};
use endurance_racing_planner_common::RacePlanner;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(event: Request) -> Result<impl IntoResponse, Error> {
    Ok(match event.body() {
        Body::Text(json) => {
            let plan = serde_json::from_str::<RacePlanner>(json);
            
            match plan {
                Ok(p) => {
                    let new_plan = RacePlanner::new(p);
                    
                    Response::builder()
                        .status(201)
                        .header("Location", format!("/plans/{}", new_plan.id.unwrap()))
                        .body(serde_json::to_string(&new_plan).unwrap())
                        .expect("failed to render response")
                },
                Err(e) => {
                    Response::builder()
                        .status(400)
                        .body(e.to_string())
                        .expect("failed to render response")
                }
            }
            
        },
        _ => Response::builder()
            .status(400)
            .body("Invalid body type".into())
            .expect("failed to render response"),
    })
}