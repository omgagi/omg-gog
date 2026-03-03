//! Classroom API request/response types.
//! All types use camelCase serde rename and flatten for forward compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------
// Course types
// ---------------------------------------------------------------

/// A Google Classroom course.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: Option<String>,
    pub name: Option<String>,
    pub owner_id: Option<String>,
    pub course_state: Option<String>,
    pub section: Option<String>,
    pub description: Option<String>,
    pub alternate_link: Option<String>,
    pub enrollment_code: Option<String>,
    pub teacher_group_email: Option<String>,
    pub course_group_email: Option<String>,
    pub calendar_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Course list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseListResponse {
    #[serde(default)]
    pub courses: Vec<Course>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// User profile types
// ---------------------------------------------------------------

/// A user profile in Classroom.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: Option<String>,
    pub name: Option<Name>,
    pub email_address: Option<String>,
    pub photo_url: Option<String>,
    #[serde(default)]
    pub permissions: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A person's name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub full_name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Student types
// ---------------------------------------------------------------

/// A student enrolled in a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Student {
    pub user_id: Option<String>,
    pub profile: Option<UserProfile>,
    pub course_id: Option<String>,
    pub student_work_folder: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Student list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentListResponse {
    #[serde(default)]
    pub students: Vec<Student>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Teacher types
// ---------------------------------------------------------------

/// A teacher in a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Teacher {
    pub user_id: Option<String>,
    pub profile: Option<UserProfile>,
    pub course_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Teacher list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherListResponse {
    #[serde(default)]
    pub teachers: Vec<Teacher>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// CourseWork types
// ---------------------------------------------------------------

/// A coursework assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseWork {
    pub course_id: Option<String>,
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub work_type: Option<String>,
    pub due_date: Option<serde_json::Value>,
    pub due_time: Option<serde_json::Value>,
    pub max_points: Option<f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// CourseWork list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseWorkListResponse {
    #[serde(default)]
    pub course_work: Vec<CourseWork>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Course material types
// ---------------------------------------------------------------

/// A course material (courseWorkMaterial).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseMaterial {
    pub course_id: Option<String>,
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub topic_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Course material list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseMaterialListResponse {
    #[serde(default)]
    pub course_work_material: Vec<CourseMaterial>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Submission types
// ---------------------------------------------------------------

/// A student submission for coursework.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentSubmission {
    pub course_id: Option<String>,
    pub course_work_id: Option<String>,
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub state: Option<String>,
    pub assigned_grade: Option<f64>,
    pub draft_grade: Option<f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Submission list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionListResponse {
    #[serde(default)]
    pub student_submissions: Vec<StudentSubmission>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Announcement types
// ---------------------------------------------------------------

/// A course announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub course_id: Option<String>,
    pub id: Option<String>,
    pub text: Option<String>,
    pub state: Option<String>,
    pub creator_user_id: Option<String>,
    pub alternate_link: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Announcement list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementListResponse {
    #[serde(default)]
    pub announcements: Vec<Announcement>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Topic types
// ---------------------------------------------------------------

/// A course topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub course_id: Option<String>,
    pub topic_id: Option<String>,
    pub name: Option<String>,
    pub update_time: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Topic list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicListResponse {
    #[serde(default)]
    pub topic: Vec<Topic>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Invitation types
// ---------------------------------------------------------------

/// An invitation to join a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invitation {
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub course_id: Option<String>,
    pub role: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Invitation list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationListResponse {
    #[serde(default)]
    pub invitations: Vec<Invitation>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------
// Guardian types
// ---------------------------------------------------------------

/// A guardian linked to a student.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Guardian {
    pub student_id: Option<String>,
    pub guardian_id: Option<String>,
    pub guardian_profile: Option<UserProfile>,
    pub invited_email_address: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Guardian list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardianListResponse {
    #[serde(default)]
    pub guardians: Vec<Guardian>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A guardian invitation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardianInvitation {
    pub student_id: Option<String>,
    pub invitation_id: Option<String>,
    pub invited_email_address: Option<String>,
    pub state: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Guardian invitation list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardianInvitationListResponse {
    #[serde(default)]
    pub guardian_invitations: Vec<GuardianInvitation>,
    pub next_page_token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // REQ-CLASS-001 (Must): Course type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: Course deserializes from Classroom API JSON
    #[test]
    fn req_class_001_course_deserialize() {
        let json_str = r#"{
            "id": "12345",
            "name": "Math 101",
            "ownerId": "teacher123",
            "courseState": "ACTIVE",
            "section": "Section A",
            "description": "Intro to Mathematics",
            "alternateLink": "https://classroom.google.com/c/12345",
            "enrollmentCode": "abc123",
            "teacherGroupEmail": "teachers@example.com",
            "courseGroupEmail": "course@example.com",
            "calendarId": "cal123"
        }"#;
        let course: Course = serde_json::from_str(json_str).unwrap();
        assert_eq!(course.id, Some("12345".to_string()));
        assert_eq!(course.name, Some("Math 101".to_string()));
        assert_eq!(course.owner_id, Some("teacher123".to_string()));
        assert_eq!(course.course_state, Some("ACTIVE".to_string()));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Acceptance: CourseListResponse deserializes with pagination
    #[test]
    fn req_class_001_course_list_response_deserialize() {
        let json_str = r#"{
            "courses": [
                {"id": "c1", "name": "Course 1"},
                {"id": "c2", "name": "Course 2"}
            ],
            "nextPageToken": "token_abc"
        }"#;
        let resp: CourseListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.courses.len(), 2);
        assert_eq!(resp.courses[0].id, Some("c1".to_string()));
        assert_eq!(resp.next_page_token, Some("token_abc".to_string()));
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Edge case: CourseListResponse with empty courses
    #[test]
    fn req_class_001_course_list_response_empty() {
        let json_str = r#"{}"#;
        let resp: CourseListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.courses.is_empty());
        assert!(resp.next_page_token.is_none());
    }

    // Requirement: REQ-CLASS-001 (Must)
    // Edge case: Course with unknown fields preserved via flatten
    #[test]
    fn req_class_001_course_unknown_fields_preserved() {
        let json_str = r#"{
            "id": "c1",
            "unknownField": "should be preserved"
        }"#;
        let course: Course = serde_json::from_str(json_str).unwrap();
        assert_eq!(course.id, Some("c1".to_string()));
        assert!(course.extra.contains_key("unknownField"));
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-002 (Must): Student type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-002 (Must)
    // Acceptance: Student deserializes from Classroom API JSON
    #[test]
    fn req_class_002_student_deserialize() {
        let json_str = r#"{
            "userId": "student123",
            "courseId": "course456",
            "profile": {
                "id": "student123",
                "name": {"givenName": "John", "familyName": "Doe", "fullName": "John Doe"},
                "emailAddress": "john@example.com"
            }
        }"#;
        let student: Student = serde_json::from_str(json_str).unwrap();
        assert_eq!(student.user_id, Some("student123".to_string()));
        assert_eq!(student.course_id, Some("course456".to_string()));
        let profile = student.profile.unwrap();
        assert_eq!(profile.email_address, Some("john@example.com".to_string()));
        let name = profile.name.unwrap();
        assert_eq!(name.given_name, Some("John".to_string()));
    }

    // Requirement: REQ-CLASS-002 (Must)
    // Acceptance: StudentListResponse deserializes
    #[test]
    fn req_class_002_student_list_response_deserialize() {
        let json_str = r#"{
            "students": [
                {"userId": "s1", "courseId": "c1"},
                {"userId": "s2", "courseId": "c1"}
            ],
            "nextPageToken": "next123"
        }"#;
        let resp: StudentListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.students.len(), 2);
        assert_eq!(resp.next_page_token, Some("next123".to_string()));
    }

    // Requirement: REQ-CLASS-002 (Must)
    // Edge case: Empty student list
    #[test]
    fn req_class_002_student_list_empty() {
        let json_str = r#"{}"#;
        let resp: StudentListResponse = serde_json::from_str(json_str).unwrap();
        assert!(resp.students.is_empty());
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-003 (Must): Teacher type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-003 (Must)
    // Acceptance: Teacher deserializes
    #[test]
    fn req_class_003_teacher_deserialize() {
        let json_str = r#"{
            "userId": "teacher123",
            "courseId": "course456",
            "profile": {
                "id": "teacher123",
                "emailAddress": "teacher@example.com"
            }
        }"#;
        let teacher: Teacher = serde_json::from_str(json_str).unwrap();
        assert_eq!(teacher.user_id, Some("teacher123".to_string()));
        assert_eq!(teacher.course_id, Some("course456".to_string()));
    }

    // Requirement: REQ-CLASS-003 (Must)
    // Acceptance: TeacherListResponse deserializes
    #[test]
    fn req_class_003_teacher_list_response_deserialize() {
        let json_str = r#"{
            "teachers": [{"userId": "t1", "courseId": "c1"}],
            "nextPageToken": "nextT"
        }"#;
        let resp: TeacherListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.teachers.len(), 1);
        assert_eq!(resp.next_page_token, Some("nextT".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-005 (Must): CourseWork type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWork deserializes
    #[test]
    fn req_class_005_coursework_deserialize() {
        let json_str = r#"{
            "courseId": "c1",
            "id": "cw1",
            "title": "Homework 1",
            "description": "Complete exercises",
            "state": "PUBLISHED",
            "workType": "ASSIGNMENT",
            "maxPoints": 100.0
        }"#;
        let cw: CourseWork = serde_json::from_str(json_str).unwrap();
        assert_eq!(cw.id, Some("cw1".to_string()));
        assert_eq!(cw.title, Some("Homework 1".to_string()));
        assert_eq!(cw.max_points, Some(100.0));
    }

    // Requirement: REQ-CLASS-005 (Must)
    // Acceptance: CourseWorkListResponse deserializes
    #[test]
    fn req_class_005_coursework_list_response_deserialize() {
        let json_str = r#"{
            "courseWork": [
                {"courseId": "c1", "id": "cw1", "title": "HW1"}
            ],
            "nextPageToken": "nextCW"
        }"#;
        let resp: CourseWorkListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.course_work.len(), 1);
        assert_eq!(resp.next_page_token, Some("nextCW".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-006 (Must): CourseMaterial type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: CourseMaterial deserializes
    #[test]
    fn req_class_006_course_material_deserialize() {
        let json_str = r#"{
            "courseId": "c1",
            "id": "m1",
            "title": "Reading Material",
            "description": "Chapter 1",
            "state": "PUBLISHED",
            "topicId": "t1"
        }"#;
        let mat: CourseMaterial = serde_json::from_str(json_str).unwrap();
        assert_eq!(mat.id, Some("m1".to_string()));
        assert_eq!(mat.title, Some("Reading Material".to_string()));
        assert_eq!(mat.topic_id, Some("t1".to_string()));
    }

    // Requirement: REQ-CLASS-006 (Must)
    // Acceptance: CourseMaterialListResponse deserializes
    #[test]
    fn req_class_006_course_material_list_response_deserialize() {
        let json_str = r#"{
            "courseWorkMaterial": [
                {"courseId": "c1", "id": "m1", "title": "Material 1"}
            ],
            "nextPageToken": "nextM"
        }"#;
        let resp: CourseMaterialListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.course_work_material.len(), 1);
        assert_eq!(resp.next_page_token, Some("nextM".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-007 (Must): StudentSubmission type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: StudentSubmission deserializes
    #[test]
    fn req_class_007_submission_deserialize() {
        let json_str = r#"{
            "courseId": "c1",
            "courseWorkId": "cw1",
            "id": "sub1",
            "userId": "s1",
            "state": "TURNED_IN",
            "assignedGrade": 95.0,
            "draftGrade": 90.0
        }"#;
        let sub: StudentSubmission = serde_json::from_str(json_str).unwrap();
        assert_eq!(sub.id, Some("sub1".to_string()));
        assert_eq!(sub.state, Some("TURNED_IN".to_string()));
        assert_eq!(sub.assigned_grade, Some(95.0));
    }

    // Requirement: REQ-CLASS-007 (Must)
    // Acceptance: SubmissionListResponse deserializes
    #[test]
    fn req_class_007_submission_list_response_deserialize() {
        let json_str = r#"{
            "studentSubmissions": [
                {"courseId": "c1", "courseWorkId": "cw1", "id": "sub1", "userId": "s1", "state": "NEW"}
            ],
            "nextPageToken": "nextSub"
        }"#;
        let resp: SubmissionListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.student_submissions.len(), 1);
        assert_eq!(resp.next_page_token, Some("nextSub".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-008 (Must): Announcement type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: Announcement deserializes
    #[test]
    fn req_class_008_announcement_deserialize() {
        let json_str = r#"{
            "courseId": "c1",
            "id": "ann1",
            "text": "Welcome to class!",
            "state": "PUBLISHED",
            "creatorUserId": "teacher1",
            "alternateLink": "https://classroom.google.com/c/c1/p/ann1"
        }"#;
        let ann: Announcement = serde_json::from_str(json_str).unwrap();
        assert_eq!(ann.id, Some("ann1".to_string()));
        assert_eq!(ann.text, Some("Welcome to class!".to_string()));
    }

    // Requirement: REQ-CLASS-008 (Must)
    // Acceptance: AnnouncementListResponse deserializes
    #[test]
    fn req_class_008_announcement_list_response_deserialize() {
        let json_str = r#"{
            "announcements": [
                {"courseId": "c1", "id": "ann1", "text": "Hello"}
            ],
            "nextPageToken": "nextAnn"
        }"#;
        let resp: AnnouncementListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.announcements.len(), 1);
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-009 (Must): Topic type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: Topic deserializes
    #[test]
    fn req_class_009_topic_deserialize() {
        let json_str = r#"{
            "courseId": "c1",
            "topicId": "topic1",
            "name": "Week 1",
            "updateTime": "2024-01-15T14:30:00Z"
        }"#;
        let topic: Topic = serde_json::from_str(json_str).unwrap();
        assert_eq!(topic.topic_id, Some("topic1".to_string()));
        assert_eq!(topic.name, Some("Week 1".to_string()));
    }

    // Requirement: REQ-CLASS-009 (Must)
    // Acceptance: TopicListResponse deserializes
    #[test]
    fn req_class_009_topic_list_response_deserialize() {
        let json_str = r#"{
            "topic": [
                {"courseId": "c1", "topicId": "t1", "name": "Week 1"}
            ],
            "nextPageToken": "nextTopic"
        }"#;
        let resp: TopicListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.topic.len(), 1);
        assert_eq!(resp.next_page_token, Some("nextTopic".to_string()));
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-010 (Must): Invitation type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: Invitation deserializes
    #[test]
    fn req_class_010_invitation_deserialize() {
        let json_str = r#"{
            "id": "inv1",
            "userId": "user123",
            "courseId": "c1",
            "role": "STUDENT"
        }"#;
        let inv: Invitation = serde_json::from_str(json_str).unwrap();
        assert_eq!(inv.id, Some("inv1".to_string()));
        assert_eq!(inv.role, Some("STUDENT".to_string()));
    }

    // Requirement: REQ-CLASS-010 (Must)
    // Acceptance: InvitationListResponse deserializes
    #[test]
    fn req_class_010_invitation_list_response_deserialize() {
        let json_str = r#"{
            "invitations": [
                {"id": "inv1", "userId": "u1", "courseId": "c1", "role": "STUDENT"}
            ],
            "nextPageToken": "nextInv"
        }"#;
        let resp: InvitationListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.invitations.len(), 1);
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-011 (Must): Guardian type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-011 (Must)
    // Acceptance: Guardian deserializes
    #[test]
    fn req_class_011_guardian_deserialize() {
        let json_str = r#"{
            "studentId": "student1",
            "guardianId": "guardian1",
            "guardianProfile": {
                "id": "guardian1",
                "emailAddress": "parent@example.com"
            },
            "invitedEmailAddress": "parent@example.com"
        }"#;
        let guardian: Guardian = serde_json::from_str(json_str).unwrap();
        assert_eq!(guardian.student_id, Some("student1".to_string()));
        assert_eq!(guardian.guardian_id, Some("guardian1".to_string()));
        assert_eq!(
            guardian.invited_email_address,
            Some("parent@example.com".to_string())
        );
    }

    // Requirement: REQ-CLASS-011 (Must)
    // Acceptance: GuardianListResponse deserializes
    #[test]
    fn req_class_011_guardian_list_response_deserialize() {
        let json_str = r#"{
            "guardians": [
                {"studentId": "s1", "guardianId": "g1"}
            ],
            "nextPageToken": "nextG"
        }"#;
        let resp: GuardianListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.guardians.len(), 1);
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-012 (Must): GuardianInvitation type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-012 (Must)
    // Acceptance: GuardianInvitation deserializes
    #[test]
    fn req_class_012_guardian_invitation_deserialize() {
        let json_str = r#"{
            "studentId": "student1",
            "invitationId": "ginv1",
            "invitedEmailAddress": "parent@example.com",
            "state": "PENDING"
        }"#;
        let inv: GuardianInvitation = serde_json::from_str(json_str).unwrap();
        assert_eq!(inv.invitation_id, Some("ginv1".to_string()));
        assert_eq!(inv.state, Some("PENDING".to_string()));
    }

    // Requirement: REQ-CLASS-012 (Must)
    // Acceptance: GuardianInvitationListResponse deserializes
    #[test]
    fn req_class_012_guardian_invitation_list_response_deserialize() {
        let json_str = r#"{
            "guardianInvitations": [
                {"studentId": "s1", "invitationId": "gi1", "state": "PENDING"}
            ],
            "nextPageToken": "nextGI"
        }"#;
        let resp: GuardianInvitationListResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(resp.guardian_invitations.len(), 1);
    }

    // ---------------------------------------------------------------
    // REQ-CLASS-004 (Must): UserProfile type serialization
    // ---------------------------------------------------------------

    // Requirement: REQ-CLASS-004 (Must)
    // Acceptance: UserProfile deserializes
    #[test]
    fn req_class_004_user_profile_deserialize() {
        let json_str = r#"{
            "id": "user123",
            "name": {
                "givenName": "Jane",
                "familyName": "Smith",
                "fullName": "Jane Smith"
            },
            "emailAddress": "jane@example.com",
            "photoUrl": "https://lh3.googleusercontent.com/photo",
            "permissions": [{"permission": "CREATE_COURSE"}]
        }"#;
        let profile: UserProfile = serde_json::from_str(json_str).unwrap();
        assert_eq!(profile.id, Some("user123".to_string()));
        assert_eq!(profile.email_address, Some("jane@example.com".to_string()));
        assert_eq!(profile.permissions.len(), 1);
    }

    // Requirement: REQ-CLASS-004 (Must)
    // Acceptance: Name type round-trip
    #[test]
    fn req_class_004_name_roundtrip() {
        let name = Name {
            given_name: Some("John".to_string()),
            family_name: Some("Doe".to_string()),
            full_name: Some("John Doe".to_string()),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&name).unwrap();
        let parsed: Name = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.given_name, Some("John".to_string()));
        assert_eq!(parsed.full_name, Some("John Doe".to_string()));
    }
}
