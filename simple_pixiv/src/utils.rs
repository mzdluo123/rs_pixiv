use actix_web::{http, HttpResponse};
use actix_web::http::header::{HeaderName, HeaderValue, TryIntoHeaderPair};

pub fn make_temporary_redirect<T>(target: T) ->HttpResponse
where T: TryInto<HeaderValue>, (HeaderName, T): TryIntoHeaderPair{
    return HttpResponse::TemporaryRedirect()
        .append_header((http::header::LOCATION, target))
        .finish();
}
pub fn make_permanent_redirect<T>(target: T) ->HttpResponse
    where T: TryInto<HeaderValue>, (HeaderName, T): TryIntoHeaderPair{
    return HttpResponse::PermanentRedirect()
        .append_header((http::header::LOCATION, target))
        .finish();
}
