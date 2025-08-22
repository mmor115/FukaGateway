use actix_web::{post, HttpResponse, Responder};
use crate::info_file_parser::lexer::InfoFileLexer;
use crate::info_file_parser::error::InfoFileParserError;


#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    println!("{}", req_body);
    HttpResponse::Ok().body(req_body)
}

#[post("/lex")]
async fn lex_info_file(req_body: String) -> Result<impl Responder, InfoFileParserError> {
    let tokens = InfoFileLexer::new(&req_body).lex()?;
    let mut output = String::new();
    for token in tokens {
        output.push_str(&token.to_string());
        output.push(' ');
    }
    Ok(HttpResponse::Ok().body(output))
}