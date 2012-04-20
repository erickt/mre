import std::map;
import std::map::{hashmap, str_hash};

import mongrel2::request;

type response = {
    request: request,
    code: uint,
    status: str,
    headers: hashmap<str, [str]>,
    body: [u8],
};

fn response_headers(req: request,
                    code: uint,
                    status: str,
                    headers: hashmap<str, [str]>,
                    body: [u8]) -> response {
    {
        request: req,
        code: code,
        status: status,
        headers: headers,
        body: body,
    }
}

fn response(req: request,
            code: uint,
            status: str,
            body: [u8]) -> response {
    response_headers(req, code, status, str_hash(), body)
}

fn redirect(req: request, location: str) -> response {
    let headers = str_hash();
    headers.insert("Location", [location]);

    response_headers(req, 302u, "Found", headers, [])
}

fn http_100(req: request) -> response {
    response(req, 100u, "Continue", [])
}

fn http_101(req: request) -> response {
    response(req, 101u, "Switching Protocols", [])
}

fn http_200_headers(req: request,
                    headers: hashmap<str, [str]>,
                    body: [u8]) -> response {
    response_headers(req, 200u, "OK", headers, body)
}

fn http_200(req: request, body: [u8]) -> response {
    http_200_headers(req, str_hash(), body)
}

fn http_201(req: request) -> response {
    response(req, 201u, "Created", [])
}

fn http_202(req: request) -> response {
    response(req, 202u, "Accepted", [])
}

fn http_203(req: request) -> response {
    response(req, 203u, "Non-Authoritative Information", [])
}

fn http_204(req: request) -> response {
    response(req, 204u, "No Content", [])
}

fn http_205(req: request) -> response {
    response(req, 205u, "Reset Content", [])
}

fn http_206(req: request) -> response {
    response(req, 206u, "Partial Content", [])
}

fn http_300(req: request) -> response {
    response(req, 300u, "Multiple Choices", [])
}

fn http_301(req: request) -> response {
    response(req, 301u, "Moved Permanently", [])
}

fn http_302(req: request) -> response {
    response(req, 302u, "Found", [])
}

fn http_303(req: request) -> response {
    response(req, 303u, "See Other", [])
}

fn http_304(req: request) -> response {
    response(req, 304u, "Not Modified", [])
}

fn http_305(req: request) -> response {
    response(req, 305u, "Use Proxy", [])
}

fn http_307(req: request) -> response {
    response(req, 305u, "Temporary Redirect", [])
}

fn http_400(req: request, body: [u8]) -> response {
    response(req, 400u, "Bad Request", body)
}

fn http_401(req: request) -> response {
    response(req, 401u, "Unauthorized", [])
}

fn http_402(req: request) -> response {
    response(req, 402u, "Payment Required", [])
}

fn http_403(req: request) -> response {
    response(req, 403u, "Forbidden", [])
}

fn http_404(req: request) -> response {
    response(req, 404u, "Not Found", [])
}

fn http_405(req: request) -> response {
    response(req, 405u, "Method Not Allowed", [])
}

fn http_406(req: request) -> response {
    response(req, 406u, "Not Acceptable", [])
}

fn http_407(req: request) -> response {
    response(req, 407u, "Proxy Authentication Required", [])
}

fn http_408(req: request) -> response {
    response(req, 408u, "Request Timeout", [])
}

fn http_409(req: request) -> response {
    response(req, 409u, "Conflict", [])
}

fn http_410(req: request) -> response {
    response(req, 410u, "Gone", [])
}

fn http_411(req: request) -> response {
    response(req, 411u, "Length Required", [])
}

fn http_412(req: request) -> response {
    response(req, 412u, "Precondition Failed", [])
}

fn http_413(req: request) -> response {
    response(req, 413u, "Request Entity Too Large", [])
}

fn http_414(req: request) -> response {
    response(req, 414u, "Request-URI Too Long", [])
}

fn http_415(req: request) -> response {
    response(req, 415u, "Unsupported Media Type", [])
}

fn http_416(req: request) -> response {
    response(req, 416u, "Requested Range Not Satisifiable", [])
}

fn http_417(req: request) -> response {
    response(req, 417u, "Expectation Failed", [])
}

fn http_500(req: request, body: [u8]) -> response {
    response(req, 500u, "Internal Server Error", body)
}

fn http_501(req: request) -> response {
    response(req, 501u, "Not Implemented", [])
}

fn http_502(req: request) -> response {
    response(req, 502u, "Bad Gateway", [])
}

fn http_503(req: request) -> response {
    response(req, 503u, "Service Unavailable", [])
}

fn http_504(req: request) -> response {
    response(req, 504u, "Gateway Timeout", [])
}

fn http_505(req: request) -> response {
    response(req, 505u, "HTTP Version Not Supported", [])
}
