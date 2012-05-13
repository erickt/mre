fn signup(req: @request, rep: @response, f: fn(str, str)) {
    let form = uri::decode_qs(req.body());

    let username = alt form.find("username") {
      none {
        rep.http_400(str::bytes("missing username"));
        ret;

      }
      some(usernames) { usernames[0]}
    };

    let password = alt form.find("password") {
      none {
        rep.http_400(str::bytes("missing password"));
        ret;
      }
      some(passwords) { passwords[0] }
    };

    let password_confirm = alt form.find("password_confirm") {
      none {
        rep.http_400(str::bytes("missing password confirmation"));
        ret;
      }
      some(password_confirms) { password_confirms[0] }
    };

    if password != password_confirm {
        rep.http_400(str::bytes("invalid password confirmation"));
        ret;
    }

    f(username, password)
}

fn login(req: @request, rep: @response, f: fn(str, str)) {
    let form = uri::decode_qs(req.body());

    let username = alt form.find("username") {
      none {
        rep.http_400(str::bytes("missing username"));
        ret;
      }
      some(usernames) { usernames[0] }
    };

    let password = alt form.find("password") {
      none {
        rep.http_400(str::bytes("missing password"));
        ret;
      }
      some(passwords) { passwords[0] }
    };

    f(username, password)
}

fn post(req: @request, rep: @response, f: fn(str, str)) {
    let form = uri::decode_qs(req.body());

    let title = alt form.find("title") {
      none {
        rep.http_400(str::bytes("missing title"));
        ret;
      }
      some(titles) { titles[0] }
    };

    if title.trim() == "" {
        rep.http_400(str::bytes("cannot have an empty title"));
        ret;
    }

    let body = alt form.find("body") {
      none {
        rep.http_400(str::bytes("missing body"));
        ret;
      }
      some(bodys) { bodys[0] }
    };

    if body.trim() == "" {
        rep.http_400(str::bytes("cannot have an empty body"));
        ret;
    }

    f(title, body)
}

fn comment(req: @request, rep: @response, f: fn(str)) {
    let form = uri::decode_qs(req.body());

    let body = alt form.find("body") {
      none {
        rep.http_400(str::bytes("missing body"));
        ret;
      }
      some(bodys) { bodys[0] }
    };

    if body.trim() == "" {
        rep.http_400(str::bytes("cannot have an empty body"));
        ret;
    }

    f(body)
}
