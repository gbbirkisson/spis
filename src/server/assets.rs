static ASSETS: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets");

fn find_files(name: &str) -> Vec<&include_dir::File> {
    ASSETS
        .find(name)
        .unwrap_or_else(|_| panic!("Could not find {name}"))
        .map(|f| {
            f.as_file()
                .unwrap_or_else(|| panic!("Could not convert to file: {name}"))
        })
        .collect()
}

fn create_route(content_type: &str, file: &'static include_dir::File) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .body(file.contents())
}

pub fn create_service(path: &str) -> actix_web::Scope {
    let mut scope = actix_web::web::scope(path);

    let files = vec![
        ("*.json", "application/json"),
        ("*.js", "application/javascript"),
        ("*.png", "image/png"),
        ("*.css", "text/css"),
        ("*.ttf", "font/ttf"),
        ("*.woff2", "font/woff2"),
    ];

    for (file_regex, content_type) in files {
        for file in find_files(file_regex) {
            scope = scope.route(
                &format!("/{}", file.path().display()),
                actix_web::web::get().to(move || async move { create_route(content_type, file) }),
            );
        }
    }

    scope
}
