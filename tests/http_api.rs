use http_body_util::BodyExt;
use serde_json::{json, Value};
use task_api::TestApp;
use tower::ServiceExt;

async fn call(
    app: &TestApp,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (u16, Value) {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let request = builder
        .body(axum::body::Body::from(
            body.map(|value| value.to_string()).unwrap_or_default(),
        ))
        .unwrap();
    let response = app.router.clone().oneshot(request).await.unwrap();
    let status = response.status().as_u16();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, parsed)
}

#[tokio::test]
async fn seed_creates_admin_and_james_bond_and_can_run_twice() {
    let app = TestApp::new().await;
    let (status, body) = call(&app, "POST", "/seed/users", None, Some(json!({}))).await;
    assert_eq!(status, 200);
    let users = body["users"].as_array().unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0]["full_name"], "Admin");
    assert_eq!(users[0]["email"], "admin@example.com");
    assert_eq!(users[0]["role"], "admin");
    assert!(users[0].get("hashed_password").is_none());
    assert!(users[0].get("password").is_none());
    assert_eq!(users[1]["full_name"], "James Bond");
    assert_eq!(users[1]["email"], "jamesbond@example.com");
    assert_eq!(users[1]["role"], "staff");

    let (again_status, again) = call(&app, "POST", "/seed/users", None, Some(json!({}))).await;
    assert_eq!(again_status, 200);
    assert_eq!(again["users"][0]["id"], users[0]["id"]);
    assert_eq!(again["users"][1]["id"], users[1]["id"]);
}

#[tokio::test]
async fn login_returns_a_challenge_and_records_the_verification_code() {
    let app = TestApp::new().await;
    call(&app, "POST", "/seed/users", None, Some(json!({}))).await;

    let (missing, _) = call(&app, "GET", "/dev/email-logs/latest", None, None).await;
    assert_eq!(missing, 404);

    let (status, body) = call(
        &app,
        "POST",
        "/auth/login",
        None,
        Some(json!({
            "email": "admin@example.com",
            "password": "admin-password"
        })),
    )
    .await;
    assert_eq!(status, 200);
    assert!(body.get("access_token").is_none());
    assert!(body.get("token").is_none());
    let challenge_id = body["login_challenge_id"].as_str().unwrap();
    assert!(!challenge_id.is_empty());

    let (log_status, log) = call(&app, "GET", "/dev/email-logs/latest", None, None).await;
    assert_eq!(log_status, 200);
    assert_eq!(log["to"], "admin@example.com");
    let code = log["code"].as_str().unwrap();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|ch| ch.is_ascii_digit()));
}

async fn verification_code(app: &TestApp, email: &str, password: &str) -> (String, String) {
    let (status, body) = call(
        app,
        "POST",
        "/auth/login",
        None,
        Some(json!({ "email": email, "password": password })),
    )
    .await;
    assert_eq!(status, 200);
    let challenge_id = body["login_challenge_id"].as_str().unwrap().to_string();
    let (log_status, log) = call(app, "GET", "/dev/email-logs/latest", None, None).await;
    assert_eq!(log_status, 200);
    let code = log["code"].as_str().unwrap().to_string();
    (challenge_id, code)
}

#[tokio::test]
async fn verify_issues_a_session_and_rejects_bad_codes() {
    let app = TestApp::new().await;
    call(&app, "POST", "/seed/users", None, Some(json!({}))).await;

    let (bad_password, _) = call(
        &app,
        "POST",
        "/auth/login",
        None,
        Some(json!({
            "email": "admin@example.com",
            "password": "wrong-password"
        })),
    )
    .await;
    assert_eq!(bad_password, 401);
    let (unknown, _) = call(
        &app,
        "POST",
        "/auth/login",
        None,
        Some(json!({
            "email": "missing@example.com",
            "password": "admin-password"
        })),
    )
    .await;
    assert_eq!(unknown, 401);
    let (no_log, _) = call(&app, "GET", "/dev/email-logs/latest", None, None).await;
    assert_eq!(no_log, 404);

    let (expired_challenge, expired_code) =
        verification_code(&app, "admin@example.com", "admin-password").await;
    let wrong = if expired_code == "000000" {
        "000001"
    } else {
        "000000"
    };
    let (wrong_status, _) = call(
        &app,
        "POST",
        "/auth/verify-2fa",
        None,
        Some(json!({
            "login_challenge_id": expired_challenge,
            "code": wrong
        })),
    )
    .await;
    assert_eq!(wrong_status, 401);

    app.clock.advance(chrono::Duration::minutes(5));
    let (expired_status, _) = call(
        &app,
        "POST",
        "/auth/verify-2fa",
        None,
        Some(json!({
            "login_challenge_id": expired_challenge,
            "code": expired_code
        })),
    )
    .await;
    assert_eq!(expired_status, 401);

    let (challenge_id, code) = verification_code(&app, "admin@example.com", "admin-password").await;
    let (ok_status, session) = call(
        &app,
        "POST",
        "/auth/verify-2fa",
        None,
        Some(json!({
            "login_challenge_id": challenge_id,
            "code": code
        })),
    )
    .await;
    assert_eq!(ok_status, 200);
    let token = session["access_token"].as_str().unwrap();
    assert_eq!(token.split('.').count(), 3);

    let (reused, _) = call(
        &app,
        "POST",
        "/auth/verify-2fa",
        None,
        Some(json!({
            "login_challenge_id": challenge_id,
            "code": code
        })),
    )
    .await;
    assert_eq!(reused, 401);

    let (old_challenge, old_code) =
        verification_code(&app, "jamesbond@example.com", "bond-password").await;
    let (new_challenge, new_code) =
        verification_code(&app, "jamesbond@example.com", "bond-password").await;
    let (stale, _) = call(
        &app,
        "POST",
        "/auth/verify-2fa",
        None,
        Some(json!({
            "login_challenge_id": old_challenge,
            "code": old_code
        })),
    )
    .await;
    assert_eq!(stale, 401);
    let (fresh, _) = call(
        &app,
        "POST",
        "/auth/verify-2fa",
        None,
        Some(json!({
            "login_challenge_id": new_challenge,
            "code": new_code
        })),
    )
    .await;
    assert_eq!(fresh, 200);

    let (admin_challenge, _admin_code) =
        verification_code(&app, "admin@example.com", "admin-password").await;
    let (_bond_challenge, bond_code) =
        verification_code(&app, "jamesbond@example.com", "bond-password").await;
    let (swapped, _) = call(
        &app,
        "POST",
        "/auth/verify-2fa",
        None,
        Some(json!({
            "login_challenge_id": admin_challenge,
            "code": bond_code
        })),
    )
    .await;
    assert_eq!(swapped, 401);
}

async fn session(app: &TestApp, email: &str, password: &str) -> String {
    let (challenge_id, code) = verification_code(app, email, password).await;
    let (status, body) = call(
        app,
        "POST",
        "/auth/verify-2fa",
        None,
        Some(json!({
            "login_challenge_id": challenge_id,
            "code": code
        })),
    )
    .await;
    assert_eq!(status, 200);
    body["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn admin_creates_and_updates_tasks_and_staff_cannot() {
    let app = TestApp::new().await;
    call(&app, "POST", "/seed/users", None, Some(json!({}))).await;
    let admin = session(&app, "admin@example.com", "admin-password").await;
    let bond = session(&app, "jamesbond@example.com", "bond-password").await;

    let (missing, _) = call(
        &app,
        "POST",
        "/tasks",
        None,
        Some(json!({
            "title": "Write report",
            "description": "Quarterly",
            "priority": "high"
        })),
    )
    .await;
    assert_eq!(missing, 401);

    let (forged, _) = call(
        &app,
        "POST",
        "/tasks",
        Some("not-a-token"),
        Some(json!({
            "title": "Write report",
            "description": "Quarterly",
            "priority": "high"
        })),
    )
    .await;
    assert_eq!(forged, 401);

    let task_body = json!({
        "title": "Write report",
        "description": "Quarterly",
        "priority": "high"
    });
    let (forbidden, _) = call(&app, "POST", "/tasks", Some(&bond), Some(task_body.clone())).await;
    assert_eq!(forbidden, 403);

    let (blank, _) = call(
        &app,
        "POST",
        "/tasks",
        Some(&admin),
        Some(json!({
            "title": "   ",
            "description": "Quarterly",
            "priority": "high"
        })),
    )
    .await;
    assert_eq!(blank, 400);

    let (bad_priority, _) = call(
        &app,
        "POST",
        "/tasks",
        Some(&admin),
        Some(json!({
            "title": "Write report",
            "description": "Quarterly",
            "priority": "urgent"
        })),
    )
    .await;
    assert_eq!(bad_priority, 400);

    let mut ids = Vec::new();
    for (title, priority) in [
        ("Alpha", "high"),
        ("Bravo", "medium"),
        ("Charlie", "low"),
        ("Delta", "high"),
        ("Echo", "low"),
    ] {
        app.clock.advance(chrono::Duration::seconds(1));
        let (status, task) = call(
            &app,
            "POST",
            "/tasks",
            Some(&admin),
            Some(json!({
                "title": title,
                "description": "Work item",
                "priority": priority
            })),
        )
        .await;
        assert_eq!(status, 201);
        assert_eq!(task["title"], title);
        assert_eq!(task["status"], "todo");
        assert_eq!(task["priority"], priority);
        assert!(task["assigned_to"].is_null());
        ids.push(task["id"].as_str().unwrap().to_string());
    }
    assert_eq!(ids.len(), 5);

    let (staff_update, _) = call(
        &app,
        "PATCH",
        &format!("/tasks/{}", ids[0]),
        Some(&bond),
        Some(json!({ "title": "Nope" })),
    )
    .await;
    assert_eq!(staff_update, 403);

    let (missing_task, _) = call(
        &app,
        "PATCH",
        &format!("/tasks/{}", uuid::Uuid::new_v4()),
        Some(&admin),
        Some(json!({ "title": "Missing" })),
    )
    .await;
    assert_eq!(missing_task, 404);

    let (bad_status, _) = call(
        &app,
        "PATCH",
        &format!("/tasks/{}", ids[0]),
        Some(&admin),
        Some(json!({ "status": "archived" })),
    )
    .await;
    assert_eq!(bad_status, 400);

    let (updated_status, updated) = call(
        &app,
        "PATCH",
        &format!("/tasks/{}", ids[0]),
        Some(&admin),
        Some(json!({
            "title": "Renamed",
            "description": "Updated copy",
            "status": "in_progress",
            "priority": "low"
        })),
    )
    .await;
    assert_eq!(updated_status, 200);
    assert_eq!(updated["title"], "Renamed");
    assert_eq!(updated["description"], "Updated copy");
    assert_eq!(updated["status"], "in_progress");
    assert_eq!(updated["priority"], "low");
    assert!(updated["assigned_to"].is_null());
}

#[tokio::test]
async fn james_bond_sees_three_assigned_tasks_and_the_second_read_hits_cache() {
    let app = TestApp::new().await;
    call(&app, "POST", "/seed/users", None, Some(json!({}))).await;
    let admin = session(&app, "admin@example.com", "admin-password").await;
    let bond = session(&app, "jamesbond@example.com", "bond-password").await;

    let (staff_assign, _) = call(
        &app,
        "POST",
        "/tasks/assign",
        Some(&bond),
        Some(json!({
            "task_ids": [uuid::Uuid::new_v4().to_string()],
            "assignee_email": "jamesbond@example.com"
        })),
    )
    .await;
    assert_eq!(staff_assign, 403);

    let mut ids = Vec::new();
    for (title, priority) in [
        ("Alpha", "high"),
        ("Bravo", "medium"),
        ("Charlie", "low"),
        ("Delta", "high"),
        ("Echo", "low"),
    ] {
        app.clock.advance(chrono::Duration::seconds(1));
        let (_, task) = call(
            &app,
            "POST",
            "/tasks",
            Some(&admin),
            Some(json!({
                "title": title,
                "description": "Work item",
                "priority": priority
            })),
        )
        .await;
        ids.push(task["id"].as_str().unwrap().to_string());
    }

    let (empty_ids, _) = call(
        &app,
        "POST",
        "/tasks/assign",
        Some(&admin),
        Some(json!({
            "task_ids": [],
            "assignee_email": "jamesbond@example.com"
        })),
    )
    .await;
    assert_eq!(empty_ids, 400);

    let (unknown_user, _) = call(
        &app,
        "POST",
        "/tasks/assign",
        Some(&admin),
        Some(json!({
            "task_ids": [ids[0]],
            "assignee_email": "missing@example.com"
        })),
    )
    .await;
    assert_eq!(unknown_user, 404);

    let (partial, _) = call(
        &app,
        "POST",
        "/tasks/assign",
        Some(&admin),
        Some(json!({
            "task_ids": [ids[0], uuid::Uuid::new_v4().to_string()],
            "assignee_email": "jamesbond@example.com"
        })),
    )
    .await;
    assert_eq!(partial, 404);

    let (admin_view, admin_body) =
        call(&app, "GET", "/tasks/view-my-tasks", Some(&admin), None).await;
    assert_eq!(admin_view, 200);
    assert_eq!(admin_body["summary"]["total_assigned_tasks"], 0);
    assert_eq!(admin_body["user"]["role"], "admin");
    assert!(admin_body["tasks"].as_array().unwrap().is_empty());

    let (assigned_status, assigned) = call(
        &app,
        "POST",
        "/tasks/assign",
        Some(&admin),
        Some(json!({
            "task_ids": [ids[0], ids[1], ids[2]],
            "assignee_email": "jamesbond@example.com"
        })),
    )
    .await;
    assert_eq!(assigned_status, 200);
    assert_eq!(assigned["assigned"], 3);

    let (first_status, first) = call(&app, "GET", "/tasks/view-my-tasks", Some(&bond), None).await;
    assert_eq!(first_status, 200);
    assert_eq!(first["cache"]["hit"], false);
    assert_eq!(first["user"]["email"], "jamesbond@example.com");
    assert_eq!(first["user"]["role"], "staff");
    assert_eq!(first["summary"]["total_assigned_tasks"], 3);
    let tasks = first["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0]["title"], "Alpha");
    assert_eq!(tasks[0]["priority"], "high");
    assert_eq!(tasks[0]["status"], "todo");
    assert_eq!(tasks[0]["assigned_to"], "jamesbond@example.com");
    assert_eq!(tasks[1]["title"], "Bravo");
    assert_eq!(tasks[1]["priority"], "medium");
    assert_eq!(tasks[2]["title"], "Charlie");
    assert_eq!(tasks[2]["priority"], "low");

    let (second_status, second) =
        call(&app, "GET", "/tasks/view-my-tasks", Some(&bond), None).await;
    assert_eq!(second_status, 200);
    assert_eq!(second["cache"]["hit"], true);
    assert_eq!(second["tasks"], first["tasks"]);

    app.clock.advance(chrono::Duration::seconds(1));
    let (_, extra) = call(
        &app,
        "POST",
        "/tasks",
        Some(&admin),
        Some(json!({
            "title": "Unassigned",
            "description": "Still nobody",
            "priority": "low"
        })),
    )
    .await;
    assert!(extra["assigned_to"].is_null());
    let (_, still_cached) = call(&app, "GET", "/tasks/view-my-tasks", Some(&bond), None).await;
    assert_eq!(still_cached["cache"]["hit"], true);

    let (update_status, _) = call(
        &app,
        "PATCH",
        &format!("/tasks/{}", ids[0]),
        Some(&admin),
        Some(json!({ "title": "Alpha revised" })),
    )
    .await;
    assert_eq!(update_status, 200);
    let (_, after_update) = call(&app, "GET", "/tasks/view-my-tasks", Some(&bond), None).await;
    assert_eq!(after_update["cache"]["hit"], false);
    assert_eq!(after_update["tasks"][0]["title"], "Alpha revised");
    let (_, cached_again) = call(&app, "GET", "/tasks/view-my-tasks", Some(&bond), None).await;
    assert_eq!(cached_again["cache"]["hit"], true);

    let (reassign_status, reassign) = call(
        &app,
        "POST",
        "/tasks/assign",
        Some(&admin),
        Some(json!({
            "task_ids": [ids[2]],
            "assignee_email": "admin@example.com"
        })),
    )
    .await;
    assert_eq!(reassign_status, 200);
    assert_eq!(reassign["assigned"], 1);

    let (_, bond_after) = call(&app, "GET", "/tasks/view-my-tasks", Some(&bond), None).await;
    assert_eq!(bond_after["cache"]["hit"], false);
    assert_eq!(bond_after["summary"]["total_assigned_tasks"], 2);
    let remaining: Vec<_> = bond_after["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["title"].as_str().unwrap())
        .collect();
    assert_eq!(remaining, ["Alpha revised", "Bravo"]);

    let (_, admin_after) = call(&app, "GET", "/tasks/view-my-tasks", Some(&admin), None).await;
    assert_eq!(admin_after["cache"]["hit"], false);
    assert_eq!(admin_after["summary"]["total_assigned_tasks"], 1);
    assert_eq!(admin_after["tasks"][0]["title"], "Charlie");
    assert_eq!(admin_after["tasks"][0]["assigned_to"], "admin@example.com");
}
