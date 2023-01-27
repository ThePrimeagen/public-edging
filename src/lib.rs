use leptos::{render_to_string, view};
use serde_json::json;
use view::app::{App, AppProps};
use worker::*;
// use turso::{ResultSet, CellValue};
use libsql_client::{ResultSet, CellValue};

mod turso;
mod utils;

trait Foo {
    fn some_method(&self);
}

struct Bar {
    thing: usize,
}

impl Foo for Bar {
    fn some_method(&self) {
        println!("self {}", self.thing);
    }
}

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get_async("/", |_, ctx| async move {
            /*
            let db = turso::Turso::connect(
                ctx.env.var("LIBSQL_CLIENT_URL").unwrap().to_string(),
                ctx.env.var("LIBSQL_CLIENT_USER").unwrap().to_string().as_str(),
                ctx.env.var("LIBSQL_CLIENT_PASS").unwrap().to_string().as_str(),
            );

            let db = libsql_client::Session::connect(
                    ctx.env.var("LIBSQL_CLIENT_URL").unwrap().to_string(),
                    ctx.env.var("LIBSQL_CLIENT_USER").unwrap().to_string().as_str(),
                    ctx.env.var("LIBSQL_CLIENT_PASS").unwrap().to_string().as_str(),
            );
            */
            let db = libsql_client::Session::connect_from_ctx(&ctx)?;

            // execute the damn query
            println!("executing query: create table");
            db.execute("CREATE TABLE IF NOT EXISTS counter (key TEXT, value INTEGER)")
                .await?;

            println!("executing query: select");
            let response = db
                .execute("SELECT * FROM counter WHERE key = 'turso'")
                .await?;

            let counter_value = match response {
                ResultSet::Error((msg, _)) => {
                    return Response::from_html(format!("Error: {}", msg))
                }
                ResultSet::Success((rows, _)) => {
                    if rows.rows.is_empty() {
                        _ = db
                            .execute("INSERT INTO counter (key, value) VALUES ('turso', 0)")
                            .await?;
                        return Response::from_html("Zero results for counter queries");
                    }
                    match rows.rows[0].cells.get("value") {
                        Some(Some(v)) => match v {
                            CellValue::Number(v) => *v,
                            _ => return Response::from_html("Unexpected counter value"),
                        },
                        _ => return Response::from_html("No value for 'value' column"),
                    }
                }
            };

            db.transaction([format!(
                "UPDATE counter SET value = {} WHERE key = 'turso'",
                counter_value + 1
            )])
            .await
            .ok();

            let html = render_to_string(move |cx| {
                return view! {cx,
                    <App counter=counter_value />
                };
            });

            return Response::from_html(html);
        })
        .post_async("/form/:field", |mut req, ctx| async move {
            if let Some(name) = ctx.param("field") {
                let form = req.form_data().await?;
                match form.get(name) {
                    Some(FormEntry::Field(value)) => {
                        return Response::from_json(&json!({ name: value }))
                    }
                    Some(FormEntry::File(_)) => {
                        return Response::error("`field` param in form shouldn't be a File", 422);
                    }
                    None => return Response::error("Bad Request", 400),
                }
            }

            Response::error("Bad Request", 400)
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}

