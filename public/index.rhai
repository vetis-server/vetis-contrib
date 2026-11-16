fn app(req, res) {
    res.write("Hello World!");

    res.status_code = 200;
    res.add_header("Content-Type", "application/xml");

    res
}

app(request, new_response());
