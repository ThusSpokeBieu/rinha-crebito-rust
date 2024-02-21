use ntex::{http::StatusCode, web::HttpResponse};
use serde_json::Value;
use tokio_postgres::{types::Json, Client, Config, NoTls, Statement};

use crate::transacao::Transacao;

pub struct PgConnection {
    client: Client,
    extrato: Statement,
    transacao: Statement,
}

impl PgConnection {
    pub async fn connect(config: Config) -> PgConnection {
        let (client, connection) = config
            .connect(NoTls)
            .await
            .expect("can't connect to postgresql");

        ntex::rt::spawn(async move {
            let _ = connection.await;
        });

        let extrato = client.prepare("SELECT * FROM extrato($1)").await.unwrap();
        let transacao = client
            .prepare("SELECT * from transacao($1, $2, $3, $4)")
            .await
            .unwrap();

        PgConnection {
            client,
            extrato,
            transacao,
        }
    }
}

impl PgConnection {
    pub async fn get_extratos(&self, id: i32) -> Result<HttpResponse, ()> {
        match self.client.query_one(&self.extrato, &[&id]).await {
            Ok(row) => {
                let extrato: Option<Json<Value>> = row.get(0);

                if extrato.is_none() {
                    return Ok(HttpResponse::NotFound().finish());
                }

                Ok(HttpResponse::with_body(
                    StatusCode::OK,
                    extrato.unwrap().0.into(),
                ))
            }
            Err(_) => Ok(HttpResponse::NotFound().finish()),
        }
    }

    pub async fn transaciona(&self, id: i32, transacao: Transacao) -> Result<HttpResponse, ()> {
        let row = self
            .client
            .query_one(
                &self.transacao,
                &[&id, &transacao.valor, &transacao.tipo, &transacao.descricao],
            )
            .await
            .unwrap();

        let error_code: i16 = row.get(0);
        let transacao: Option<Json<Value>> = row.get(1);
        match error_code {
            0 => Ok(HttpResponse::with_body(
                StatusCode::OK,
                transacao.unwrap().0.into(),
            )),
            1 => Ok(HttpResponse::build(StatusCode::NOT_FOUND).finish()),
            2 => Ok(HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).finish()),
            _ => Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()),
        }
    }
}
