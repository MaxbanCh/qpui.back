use crate::database::db::connect_to_database;
use serde::{Deserialize, Serialize};
use actix_web::{post, web, HttpResponse, Responder, error};

#[derive(Debug, sqlx::Type, Deserialize, Serialize, Clone, Copy)] 
#[sqlx(type_name = "category", rename_all = "PascalCase")]
pub enum Category {
    Informatique,
    Sciences,
    Histoire,
    Geographie,
    Classique,
    Moderne,
    Generale,
    Internet,
    Sport,
}

impl Category {
    fn to_string(&self) -> String {
        match self {
            Category::Informatique => "Informatique".to_string(),
            Category::Sciences => "Sciences".to_string(),
            Category::Histoire => "Histoire".to_string(),
            Category::Geographie => "Geographie".to_string(),
            Category::Classique => "Classique".to_string(),
            Category::Moderne => "Moderne".to_string(),
            Category::Generale => "Generale".to_string(),
            Category::Internet => "Internet".to_string(),
            Category::Sport => "Sport".to_string(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct Question {
    pub id: i32,
    pub question: String,
    pub answer: String,
    pub category: Category,
    pub notes: Option<String>,
    pub is_public: bool,
}

impl Question {
    pub async fn insert_into_db(
        question: String,
        answer: String,
        category: Category,
        notes: Option<String>,
        is_public: bool,
    ) -> std::result::Result<Question, sqlx::Error> {
        let pool: sqlx::PgPool = connect_to_database().await;

        let inserted_question = sqlx::query_as::<_, Question>(
            "INSERT INTO questions (question, answer, category, notes, is_public) VALUES ($1, $2, $3::category, $4, $5) RETURNING *",
        )
        .bind(question)
        .bind(answer)
        .bind(category.to_string())  
        .bind(notes)
        .bind(is_public)
        .fetch_one(&pool)
        .await
        .expect("Failed to insert question");

        println!("Inserted question: {:?}", inserted_question);
        return Ok(inserted_question);
    }

    pub async fn get_question_by_id(id: i32) -> Option<Question> {
        let pool: sqlx::PgPool = connect_to_database().await;

        sqlx::query_as::<_, Question>("SELECT * FROM questions WHERE id = $1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .expect("Failed to fetch question")
    }
}

pub async fn get_all_questions() -> Vec<Question> {
    let pool: sqlx::PgPool = connect_to_database().await;

    let questions = sqlx::query_as::<_, Question>("SELECT * FROM questions")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch questions");

    return questions;
}

pub async fn get_questions_by_category(category: Category) -> Vec<Question> {
    let pool: sqlx::PgPool = connect_to_database().await;

    let questions = sqlx::query_as::<_, Question>("SELECT * FROM questions WHERE category = $1::category")
        .bind(category.to_string())  
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch questions by category");

    return questions;
}

pub async fn set_question_to_public(id: i32) -> Option<Question> {
    let pool: sqlx::PgPool = connect_to_database().await;

    let updated_question = sqlx::query_as::<_, Question>(
        "UPDATE questions SET is_public = true WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .expect("Failed to update question");

    return updated_question;
}

pub async fn delete_question(id: i32) -> bool {
    let pool: sqlx::PgPool = connect_to_database().await;

    let result = sqlx::query("DELETE FROM questions WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("Failed to delete question");

    return result.rows_affected() > 0;
}

pub async fn update_question(
    id: i32,
    question: Option<String>,
    answer: Option<String>,
    category: Option<Category>,
    notes: Option<String>,
    is_public: Option<bool>,
) -> Option<Question> {
    let pool: sqlx::PgPool = connect_to_database().await;

    let updated_question = sqlx::query_as::<_, Question>(
        "UPDATE questions SET question = COALESCE($1, question), answer = COALESCE($2, answer), category = COALESCE($3::category, category), notes = COALESCE($4, notes), is_public = COALESCE($5, is_public) WHERE id = $6 RETURNING *",
    )
    .bind(question)
    .bind(answer)
    .bind(category.map(|c| c.to_string()))  
    .bind(notes)
    .bind(is_public)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .expect("Failed to update question");

    return updated_question;
}

pub async fn get_public_questions() -> Vec<Question> {
    let pool: sqlx::PgPool = connect_to_database().await;

    let questions = sqlx::query_as::<_, Question>("SELECT * FROM questions WHERE is_public = true")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch public questions");

    return questions;
}

// #[post("/question")]
pub async fn create_question(question: web::Json<Question>) -> impl Responder {
    println!("Received question: {:?}", question);
    match Question::insert_into_db(
        question.question.clone(),
        question.answer.clone(),
        question.category,
        question.notes.clone(),
        question.is_public,
    ).await {
        Ok(inserted_question) => HttpResponse::Ok().json(inserted_question),
        Err(e) => {
            eprintln!("Error inserting question: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to create question")
        }
    }
}