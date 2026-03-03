//! Classroom service integration tests.

use omega_google::services::classroom::courses::*;
use omega_google::services::classroom::coursework::*;
use omega_google::services::classroom::roster::*;
use omega_google::services::classroom::types::*;

// ---------------------------------------------------------------
// REQ-CLASS-001 (Must): Course deserialization from full JSON
// ---------------------------------------------------------------

// Requirement: REQ-CLASS-001 (Must)
// Acceptance: Full course structure from a realistic Classroom API response
#[test]
fn req_class_001_integration_full_course_from_api() {
    // REQ-CLASS-001
    let api_response = r#"{
        "id": "123456789",
        "name": "AP Computer Science A",
        "ownerId": "teacher001",
        "courseState": "ACTIVE",
        "section": "Period 3",
        "description": "Advanced Placement Computer Science with Java programming focus. Students will learn object-oriented programming, data structures, and algorithms.",
        "alternateLink": "https://classroom.google.com/c/123456789",
        "enrollmentCode": "xk7tm9q",
        "teacherGroupEmail": "cs_teachers@school.edu",
        "courseGroupEmail": "cs_all@school.edu",
        "calendarId": "calendar_cs_period3@group.calendar.google.com",
        "creationTime": "2024-08-15T10:00:00.000Z",
        "updateTime": "2024-09-01T14:30:00.000Z",
        "room": "Building A, Room 204"
    }"#;

    let course: Course = serde_json::from_str(api_response).unwrap();

    assert_eq!(course.id, Some("123456789".to_string()));
    assert_eq!(course.name, Some("AP Computer Science A".to_string()));
    assert_eq!(course.owner_id, Some("teacher001".to_string()));
    assert_eq!(course.course_state, Some("ACTIVE".to_string()));
    assert_eq!(course.section, Some("Period 3".to_string()));
    assert!(course
        .description
        .as_ref()
        .unwrap()
        .contains("Java programming"));
    assert_eq!(
        course.alternate_link,
        Some("https://classroom.google.com/c/123456789".to_string())
    );
    assert_eq!(course.enrollment_code, Some("xk7tm9q".to_string()));
    assert_eq!(
        course.teacher_group_email,
        Some("cs_teachers@school.edu".to_string())
    );
    assert_eq!(
        course.course_group_email,
        Some("cs_all@school.edu".to_string())
    );
    assert!(course.calendar_id.is_some());

    // Unknown fields preserved via flatten
    assert!(course.extra.contains_key("creationTime"));
    assert!(course.extra.contains_key("room"));
}

// ---------------------------------------------------------------
// REQ-CLASS-002 (Must): Student list response deserialization
// ---------------------------------------------------------------

// Requirement: REQ-CLASS-002 (Must)
// Acceptance: Full student roster from a realistic Classroom API response
#[test]
fn req_class_002_integration_student_list_from_api() {
    // REQ-CLASS-002
    let api_response = r#"{
        "students": [
            {
                "userId": "student001",
                "courseId": "123456789",
                "profile": {
                    "id": "student001",
                    "name": {
                        "givenName": "Emma",
                        "familyName": "Johnson",
                        "fullName": "Emma Johnson"
                    },
                    "emailAddress": "emma.johnson@school.edu",
                    "photoUrl": "https://lh3.googleusercontent.com/photo_emma"
                },
                "studentWorkFolder": {
                    "id": "folder001",
                    "title": "Emma Johnson - AP CS"
                }
            },
            {
                "userId": "student002",
                "courseId": "123456789",
                "profile": {
                    "id": "student002",
                    "name": {
                        "givenName": "Liam",
                        "familyName": "Williams",
                        "fullName": "Liam Williams"
                    },
                    "emailAddress": "liam.williams@school.edu"
                }
            },
            {
                "userId": "student003",
                "courseId": "123456789",
                "profile": {
                    "id": "student003",
                    "name": {
                        "givenName": "Sophia",
                        "familyName": "Martinez",
                        "fullName": "Sophia Martinez"
                    },
                    "emailAddress": "sophia.martinez@school.edu",
                    "permissions": [{"permission": "CREATE_COURSE"}]
                }
            }
        ],
        "nextPageToken": "student_page_2"
    }"#;

    let resp: StudentListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.students.len(), 3);
    assert_eq!(resp.next_page_token, Some("student_page_2".to_string()));

    // First student: full profile with work folder
    let s1 = &resp.students[0];
    assert_eq!(s1.user_id, Some("student001".to_string()));
    assert_eq!(s1.course_id, Some("123456789".to_string()));
    let p1 = s1.profile.as_ref().unwrap();
    assert_eq!(
        p1.email_address,
        Some("emma.johnson@school.edu".to_string())
    );
    let n1 = p1.name.as_ref().unwrap();
    assert_eq!(n1.given_name, Some("Emma".to_string()));
    assert_eq!(n1.family_name, Some("Johnson".to_string()));
    assert_eq!(n1.full_name, Some("Emma Johnson".to_string()));
    assert!(s1.student_work_folder.is_some());

    // Third student: has permissions
    let p3 = resp.students[2].profile.as_ref().unwrap();
    assert_eq!(p3.permissions.len(), 1);
}

// ---------------------------------------------------------------
// REQ-CLASS-003 (Must): Teacher list response deserialization
// ---------------------------------------------------------------

// Requirement: REQ-CLASS-003 (Must)
// Acceptance: Full teacher roster from a realistic Classroom API response
#[test]
fn req_class_003_integration_teacher_list_from_api() {
    // REQ-CLASS-003
    let api_response = r#"{
        "teachers": [
            {
                "userId": "teacher001",
                "courseId": "123456789",
                "profile": {
                    "id": "teacher001",
                    "name": {
                        "givenName": "Dr. Sarah",
                        "familyName": "Mitchell",
                        "fullName": "Dr. Sarah Mitchell"
                    },
                    "emailAddress": "s.mitchell@school.edu",
                    "photoUrl": "https://lh3.googleusercontent.com/photo_mitchell"
                }
            },
            {
                "userId": "teacher002",
                "courseId": "123456789",
                "profile": {
                    "id": "teacher002",
                    "name": {
                        "givenName": "James",
                        "familyName": "Lee",
                        "fullName": "James Lee"
                    },
                    "emailAddress": "j.lee@school.edu"
                }
            }
        ],
        "nextPageToken": null
    }"#;

    let resp: TeacherListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.teachers.len(), 2);
    assert!(resp.next_page_token.is_none());

    let t1 = &resp.teachers[0];
    assert_eq!(t1.user_id, Some("teacher001".to_string()));
    let p1 = t1.profile.as_ref().unwrap();
    let n1 = p1.name.as_ref().unwrap();
    assert_eq!(n1.full_name, Some("Dr. Sarah Mitchell".to_string()));
    assert_eq!(p1.email_address, Some("s.mitchell@school.edu".to_string()));

    let t2 = &resp.teachers[1];
    assert_eq!(t2.user_id, Some("teacher002".to_string()));
}

// ---------------------------------------------------------------
// REQ-CLASS-005 (Must): CourseWork deserialization
// ---------------------------------------------------------------

// Requirement: REQ-CLASS-005 (Must)
// Acceptance: Full coursework structure from a realistic Classroom API response
#[test]
fn req_class_005_integration_coursework_from_api() {
    // REQ-CLASS-005
    let api_response = r#"{
        "courseWork": [
            {
                "courseId": "123456789",
                "id": "cw001",
                "title": "Homework 1: Variables and Data Types",
                "description": "Complete exercises 1-20 from Chapter 2. Show all work.",
                "state": "PUBLISHED",
                "workType": "ASSIGNMENT",
                "dueDate": {"year": 2024, "month": 9, "day": 15},
                "dueTime": {"hours": 23, "minutes": 59},
                "maxPoints": 100.0,
                "creatorUserId": "teacher001",
                "topicId": "topic_week1"
            },
            {
                "courseId": "123456789",
                "id": "cw002",
                "title": "Quiz 1: Control Structures",
                "description": "Short answer quiz covering if/else, loops, and switch statements",
                "state": "PUBLISHED",
                "workType": "SHORT_ANSWER_QUESTION",
                "maxPoints": 50.0
            },
            {
                "courseId": "123456789",
                "id": "cw003",
                "title": "Project: Calculator App",
                "description": "Build a calculator application using OOP principles",
                "state": "DRAFT",
                "workType": "ASSIGNMENT",
                "maxPoints": 200.0
            }
        ],
        "nextPageToken": "cw_next_page"
    }"#;

    let resp: CourseWorkListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.course_work.len(), 3);
    assert_eq!(resp.next_page_token, Some("cw_next_page".to_string()));

    // First: homework assignment with due date
    let cw1 = &resp.course_work[0];
    assert_eq!(cw1.course_id, Some("123456789".to_string()));
    assert_eq!(cw1.id, Some("cw001".to_string()));
    assert_eq!(
        cw1.title,
        Some("Homework 1: Variables and Data Types".to_string())
    );
    assert!(cw1.description.as_ref().unwrap().contains("Chapter 2"));
    assert_eq!(cw1.state, Some("PUBLISHED".to_string()));
    assert_eq!(cw1.work_type, Some("ASSIGNMENT".to_string()));
    assert_eq!(cw1.max_points, Some(100.0));
    assert!(cw1.due_date.is_some());
    assert!(cw1.due_time.is_some());

    // Second: quiz
    let cw2 = &resp.course_work[1];
    assert_eq!(cw2.work_type, Some("SHORT_ANSWER_QUESTION".to_string()));
    assert_eq!(cw2.max_points, Some(50.0));

    // Third: draft project
    let cw3 = &resp.course_work[2];
    assert_eq!(cw3.state, Some("DRAFT".to_string()));
    assert_eq!(cw3.max_points, Some(200.0));
}

// ---------------------------------------------------------------
// REQ-CLASS-001 (Must): URL builder verification
// ---------------------------------------------------------------

// Requirement: REQ-CLASS-001 (Must)
// Acceptance: URL builders produce correct Classroom API URLs
#[test]
fn req_class_001_integration_url_builders() {
    // REQ-CLASS-001
    // Courses list without params
    let url = build_courses_list_url(None, None, None);
    assert_eq!(url, "https://classroom.googleapis.com/v1/courses");

    // Courses list with state filter
    let url = build_courses_list_url(Some("ACTIVE"), None, None);
    assert!(url.contains("courseStates=ACTIVE"));

    // Courses list with pagination
    let url = build_courses_list_url(None, Some(30), Some("page2_token"));
    assert!(url.contains("pageSize=30"));
    assert!(url.contains("pageToken=page2_token"));

    // Course get URL
    let url = build_course_get_url("123456789");
    assert!(url.contains("/courses/123456789"));

    // Course create URL
    let url = build_course_create_url();
    assert!(url.ends_with("/courses"));

    // Course web URL
    let url = build_course_url("123456789");
    assert_eq!(url, "https://classroom.google.com/c/123456789");
}

// ---------------------------------------------------------------
// REQ-CLASS-002 (Must): Roster URL builder verification
// ---------------------------------------------------------------

// Requirement: REQ-CLASS-002 (Must)
// Acceptance: Roster URL builders produce correct URLs
#[test]
fn req_class_002_integration_roster_url_builders() {
    // REQ-CLASS-002
    // Students list
    let url = build_students_list_url("123456789", None, None);
    assert!(url.contains("/courses/123456789/students"));
    assert!(!url.contains("?"));

    // Students list with params
    let url = build_students_list_url("123456789", Some(50), Some("next_student"));
    assert!(url.contains("pageSize=50"));
    assert!(url.contains("pageToken=next_student"));

    // Student add
    let url = build_student_add_url("123456789");
    assert!(url.contains("/courses/123456789/students"));

    // Student remove
    let url = build_student_remove_url("123456789", "student001");
    assert!(url.contains("/courses/123456789/students/student001"));

    // Teachers list
    let url = build_teachers_list_url("123456789", None, None);
    assert!(url.contains("/courses/123456789/teachers"));

    // Teacher add
    let url = build_teacher_add_url("123456789");
    assert!(url.contains("/courses/123456789/teachers"));
}

// ---------------------------------------------------------------
// REQ-CLASS-005 (Must): Coursework URL builder verification
// ---------------------------------------------------------------

// Requirement: REQ-CLASS-005 (Must)
// Acceptance: Coursework URL builders produce correct URLs
#[test]
fn req_class_005_integration_coursework_url_builders() {
    // REQ-CLASS-005
    // Coursework list
    let url = build_coursework_list_url("123456789", None, None);
    assert!(url.contains("/courses/123456789/courseWork"));
    assert!(!url.contains("?"));

    // Coursework list with params
    let url = build_coursework_list_url("123456789", Some(20), Some("cw_page2"));
    assert!(url.contains("pageSize=20"));
    assert!(url.contains("pageToken=cw_page2"));

    // Coursework get
    let url = build_coursework_get_url("123456789", "cw001");
    assert!(url.contains("/courses/123456789/courseWork/cw001"));

    // Coursework create
    let url = build_coursework_create_url("123456789");
    assert!(url.contains("/courses/123456789/courseWork"));

    // Coursework create body
    let body = build_coursework_create_body(
        "Final Exam",
        "ASSIGNMENT",
        Some("Comprehensive final exam covering all topics"),
        Some(300.0),
        Some("DRAFT"),
    );
    assert_eq!(body["title"], "Final Exam");
    assert_eq!(body["workType"], "ASSIGNMENT");
    assert_eq!(
        body["description"],
        "Comprehensive final exam covering all topics"
    );
    assert_eq!(body["maxPoints"], 300.0);
    assert_eq!(body["state"], "DRAFT");
}

// ---------------------------------------------------------------
// REQ-CLASS-001 (Must): Course list response from API
// ---------------------------------------------------------------

// Requirement: REQ-CLASS-001 (Must)
// Acceptance: CourseListResponse with multiple courses
#[test]
fn req_class_001_integration_course_list_from_api() {
    // REQ-CLASS-001
    let api_response = r#"{
        "courses": [
            {
                "id": "c001",
                "name": "AP Computer Science A",
                "ownerId": "teacher001",
                "courseState": "ACTIVE",
                "section": "Period 3",
                "enrollmentCode": "abc123"
            },
            {
                "id": "c002",
                "name": "Data Structures",
                "ownerId": "teacher001",
                "courseState": "ACTIVE",
                "section": "Period 5"
            },
            {
                "id": "c003",
                "name": "Intro to Programming",
                "ownerId": "teacher002",
                "courseState": "ARCHIVED"
            }
        ],
        "nextPageToken": "course_next_page"
    }"#;

    let resp: CourseListResponse = serde_json::from_str(api_response).unwrap();

    assert_eq!(resp.courses.len(), 3);
    assert_eq!(resp.next_page_token, Some("course_next_page".to_string()));

    assert_eq!(
        resp.courses[0].name,
        Some("AP Computer Science A".to_string())
    );
    assert_eq!(resp.courses[0].course_state, Some("ACTIVE".to_string()));
    assert_eq!(resp.courses[2].course_state, Some("ARCHIVED".to_string()));
}
