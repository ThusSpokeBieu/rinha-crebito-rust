use regex::Regex;
use tok_io::sync::OnceCell;

static PATH_REGEX: OnceCell<regex::Regex> = OnceCell::const_new();

pub async fn path_regex() -> &'static regex::Regex {
    PATH_REGEX
        .get_or_init(|| async { Regex::new(r"/clientes/([1-5])/(extrato|transacoes)").unwrap() })
        .await
}
