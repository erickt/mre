fn signup(req: @request<app::data>, rep: @response, f: fn(@str, @str)) {
    let form = uri::decode_form_urlencoded(*req.body());

    let username = alt form.find("username") {
      none {
        rep.reply_http(400u, "missing username");
        ret;

      }
      some(usernames) { (*usernames)[0u] }
    };

    let password = alt form.find("password") {
      none {
        rep.reply_http(400u, "missing password");
        ret;
      }
      some(passwords) { (*passwords)[0u] }
    };

    let password_confirm = alt form.find("password_confirm") {
      none {
        rep.reply_http(400u, "missing password confirmation");
        ret;
      }
      some(password_confirms) { (*password_confirms)[0u] }
    };

    if password != password_confirm {
        rep.reply_http(400u, "invalid password confirmation");
        ret;
    }

    f(username, password)
}

fn login(req: @request<app::data>, rep: @response, f: fn(@str, @str)) {
    let form = uri::decode_form_urlencoded(*req.body());

    let username = alt form.find("username") {
      none {
        rep.reply_http(400u, "missing username");
        ret;
      }
      some(usernames) { (*usernames)[0u] }
    };

    let password = alt form.find("password") {
      none {
        rep.reply_http(400u, "missing password");
        ret;
      }
      some(passwords) { (*passwords)[0u] }
    };

    f(username, password)
}

fn post(req: @request<app::data>, rep: @response, f: fn(@str, @str)) {
    let form = uri::decode_form_urlencoded(*req.body());

    let title = alt form.find("title") {
      none {
        rep.reply_http(400u, "missing title");
        ret;
      }
      some(titles) { (*titles)[0u] }
    };

    if (*title).trim() == "" {
        rep.reply_http(400u, "cannot have an empty title");
        ret;
    }

    let body = alt form.find("body") {
      none {
        rep.reply_http(400u, "missing body");
        ret;
      }
      some(bodys) { (*bodys)[0u] }
    };

    if (*body).trim() == "" {
        rep.reply_http(400u, "cannot have an empty body");
        ret;
    }

    f(title, body)
}

fn comment(req: @request<app::data>, rep: @response, f: fn(@str)) {
    let form = uri::decode_form_urlencoded(*req.body());

    let body = alt form.find("body") {
      none {
        rep.reply_http(400u, "missing body");
        ret;
      }
      some(bodys) { (*bodys)[0u] }
    };

    if (*body).trim() == "" {
        rep.reply_http(400u, "cannot have an empty body");
        ret;
    }

    f(body)
}
