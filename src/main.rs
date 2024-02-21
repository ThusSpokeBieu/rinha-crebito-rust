use std::env;

use ntex::service::{Service, ServiceCtx, ServiceFactory};
use ntex::time::Seconds;
use ntex::util::PoolId;
use ntex::{
    http::{HttpService, KeepAlive, Request, Response, StatusCode},
    web::{Error, HttpResponse},
};
use ntex_bytes::BytesMut;
use tokio_postgres::Config;
use transacao::Transacao;
use validator::Validate;

mod db;
mod transacao;
mod utils;

struct App(db::PgConnection);

impl Service<Request> for App {
    type Response = Response;
    type Error = Error;

    async fn call(&self, mut req: Request, _: ServiceCtx<'_, Self>) -> Result<Response, Error> {
        let caps = utils::path_regex().await.captures(req.path());

        if caps.is_none() {
            return Ok(HttpResponse::NotFound().finish());
        }

        let caps = caps.unwrap();
        let id: i32 = caps[1].parse().unwrap();

        match &caps[2] {
            "extrato" => match self.0.get_extratos(id).await {
                Ok(ok) => Ok(ok),
                Err(_) => Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()),
            },
            "transacoes" => {
                let mut bytes = BytesMut::new();
                while let Some(chunk) = req.payload().recv().await {
                    let chunk = chunk?;
                    bytes.extend_from_slice(&chunk);
                }

                let transacao = simd_json::serde::from_slice::<Transacao>(bytes.as_mut());

                if transacao.is_err() {
                    return Ok(HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).finish());
                }

                let transacao = transacao.unwrap();

                match transacao.validate() {
                    Ok(_) => Ok(self.0.transaciona(id, transacao).await.unwrap()),
                    Err(_) => Ok(HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).finish()),
                }
            }
            _ => Ok(HttpResponse::NotFound().finish()),
        }
    }
}

struct AppFactory;

impl ServiceFactory<Request> for AppFactory {
    type Response = Response;
    type Error = Error;
    type Service = App;
    type InitError = ();

    async fn create(&self, _: ()) -> Result<Self::Service, Self::InitError> {
        let mut config = Config::new();
        config.host_path("/var/run/postgresql");
        config.user("rinha");
        config.password("rinha");
        config.dbname("crebito");

        Ok(App(db::PgConnection::connect(config).await))
    }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
        .expect("PORT não definida")
        .parse()
        .expect("PORT não é um número válido");

    println!("Tô te ouvindo filé na porta {} 😎 🔥🔥 🚀🚀", port);

    let host = format!("0.0.0.0:{}", port);

    ntex::server::build()
        .backlog(1024)
        .bind("rinha", host, |cfg| {
            cfg.memory_pool(PoolId::P1);
            PoolId::P1.set_read_params(65535, 2048);
            PoolId::P1.set_write_params(65535, 2048);

            HttpService::build()
                .keep_alive(KeepAlive::Os)
                .client_timeout(Seconds(0))
                .headers_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                .payload_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                .h1(AppFactory)
        })?
        .workers(1)
        .run()
        .await
}
