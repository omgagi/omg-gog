//! Classroom CLI subcommand tree (clap derive).

use clap::{Args, Subcommand};

/// Google Classroom service commands.
#[derive(Args, Debug)]
pub struct ClassroomArgs {
    #[command(subcommand)]
    pub command: ClassroomCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomCommand {
    /// Course operations
    Courses(ClassroomCoursesArgs),
    /// Student roster operations
    Students(ClassroomStudentsArgs),
    /// Teacher roster operations
    Teachers(ClassroomTeachersArgs),
    /// Roster operations (combined students and teachers)
    Roster(ClassroomRosterArgs),
    /// Coursework operations
    Coursework(ClassroomCourseworkArgs),
    /// Course materials operations
    Materials(ClassroomMaterialsArgs),
    /// Student submission operations
    Submissions(ClassroomSubmissionsArgs),
    /// Announcement operations
    Announcements(ClassroomAnnouncementsArgs),
    /// Topic operations
    Topics(ClassroomTopicsArgs),
    /// Invitation operations
    Invitations(ClassroomInvitationsArgs),
    /// Guardian operations
    Guardians(ClassroomGuardiansArgs),
    /// Guardian invitation operations
    GuardianInvitations(ClassroomGuardianInvitationsArgs),
    /// User profile
    Profile(ClassroomProfileArgs),
}

// ---------------------------------------------------------------
// Courses
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomCoursesArgs {
    #[command(subcommand)]
    pub command: ClassroomCoursesCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomCoursesCommand {
    /// List courses
    List(ClassroomCoursesListArgs),
    /// Get course details
    Get(ClassroomCoursesGetArgs),
    /// Create a new course
    Create(ClassroomCoursesCreateArgs),
    /// Update a course
    Update(ClassroomCoursesUpdateArgs),
    /// Delete a course
    Delete(ClassroomCoursesDeleteArgs),
    /// Archive a course
    Archive(ClassroomCoursesArchiveArgs),
    /// Unarchive a course
    Unarchive(ClassroomCoursesUnarchiveArgs),
    /// Join a course
    Join(ClassroomCoursesJoinArgs),
    /// Leave a course
    Leave(ClassroomCoursesLeaveArgs),
    /// Get course web URL
    Url(ClassroomCoursesUrlArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesListArgs {
    /// Filter by course state (ACTIVE, ARCHIVED, PROVISIONED, DECLINED, SUSPENDED)
    #[arg(long)]
    pub state: Option<String>,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesGetArgs {
    /// Course ID
    pub course_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesCreateArgs {
    /// Course name
    pub name: String,
    /// Owner email
    #[arg(long)]
    pub owner: Option<String>,
    /// Course state
    #[arg(long)]
    pub state: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesUpdateArgs {
    /// Course ID
    pub course_id: String,
    /// New name
    #[arg(long)]
    pub name: Option<String>,
    /// New state
    #[arg(long)]
    pub state: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesDeleteArgs {
    /// Course ID
    pub course_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesArchiveArgs {
    /// Course ID
    pub course_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesUnarchiveArgs {
    /// Course ID
    pub course_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesJoinArgs {
    /// Course ID or enrollment code
    pub course_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesLeaveArgs {
    /// Course ID
    pub course_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomCoursesUrlArgs {
    /// Course IDs
    pub course_ids: Vec<String>,
}

// ---------------------------------------------------------------
// Students
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomStudentsArgs {
    #[command(subcommand)]
    pub command: ClassroomStudentsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomStudentsCommand {
    /// List students in a course
    List(ClassroomStudentsListArgs),
    /// Get a student
    Get(ClassroomStudentsGetArgs),
    /// Add a student to a course
    Add(ClassroomStudentsAddArgs),
    /// Remove a student from a course
    Remove(ClassroomStudentsRemoveArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomStudentsListArgs {
    /// Course ID
    pub course_id: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomStudentsGetArgs {
    /// Course ID
    pub course_id: String,
    /// Student user ID
    pub user_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomStudentsAddArgs {
    /// Course ID
    pub course_id: String,
    /// Student user ID or email
    pub user_id: String,
    /// Enrollment code
    #[arg(long)]
    pub enrollment_code: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomStudentsRemoveArgs {
    /// Course ID
    pub course_id: String,
    /// Student user ID
    pub user_id: String,
}

// ---------------------------------------------------------------
// Teachers
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomTeachersArgs {
    #[command(subcommand)]
    pub command: ClassroomTeachersCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomTeachersCommand {
    /// List teachers in a course
    List(ClassroomTeachersListArgs),
    /// Get a teacher
    Get(ClassroomTeachersGetArgs),
    /// Add a teacher to a course
    Add(ClassroomTeachersAddArgs),
    /// Remove a teacher from a course
    Remove(ClassroomTeachersRemoveArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomTeachersListArgs {
    /// Course ID
    pub course_id: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomTeachersGetArgs {
    /// Course ID
    pub course_id: String,
    /// Teacher user ID
    pub user_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomTeachersAddArgs {
    /// Course ID
    pub course_id: String,
    /// Teacher user ID or email
    pub user_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomTeachersRemoveArgs {
    /// Course ID
    pub course_id: String,
    /// Teacher user ID
    pub user_id: String,
}

// ---------------------------------------------------------------
// Roster (combined view)
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomRosterArgs {
    #[command(subcommand)]
    pub command: ClassroomRosterCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomRosterCommand {
    /// Show full roster (students + teachers)
    List(ClassroomRosterListArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomRosterListArgs {
    /// Course ID
    pub course_id: String,
    /// Show only students
    #[arg(long)]
    pub students: bool,
    /// Show only teachers
    #[arg(long)]
    pub teachers: bool,
}

// ---------------------------------------------------------------
// Coursework
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomCourseworkArgs {
    #[command(subcommand)]
    pub command: ClassroomCourseworkCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomCourseworkCommand {
    /// List coursework
    List(ClassroomCourseworkListArgs),
    /// Get coursework details
    Get(ClassroomCourseworkGetArgs),
    /// Create coursework
    Create(ClassroomCourseworkCreateArgs),
    /// Update coursework
    Update(ClassroomCourseworkUpdateArgs),
    /// Delete coursework
    Delete(ClassroomCourseworkDeleteArgs),
    /// Modify assignees
    Assignees(ClassroomCourseworkAssigneesArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomCourseworkListArgs {
    /// Course ID
    pub course_id: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomCourseworkGetArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomCourseworkCreateArgs {
    /// Course ID
    pub course_id: String,
    /// Title
    #[arg(long)]
    pub title: String,
    /// Work type (ASSIGNMENT, SHORT_ANSWER_QUESTION, MULTIPLE_CHOICE_QUESTION)
    #[arg(long, default_value = "ASSIGNMENT")]
    pub work_type: String,
    /// Description
    #[arg(long)]
    pub description: Option<String>,
    /// Max points
    #[arg(long)]
    pub max_points: Option<f64>,
    /// State (PUBLISHED, DRAFT)
    #[arg(long)]
    pub state: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomCourseworkUpdateArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
    /// New title
    #[arg(long)]
    pub title: Option<String>,
    /// New description
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomCourseworkDeleteArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomCourseworkAssigneesArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
    /// Student IDs to assign
    #[arg(long)]
    pub add: Vec<String>,
}

// ---------------------------------------------------------------
// Materials
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomMaterialsArgs {
    #[command(subcommand)]
    pub command: ClassroomMaterialsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomMaterialsCommand {
    /// List materials
    List(ClassroomMaterialsListArgs),
    /// Get material details
    Get(ClassroomMaterialsGetArgs),
    /// Create a material
    Create(ClassroomMaterialsCreateArgs),
    /// Update a material
    Update(ClassroomMaterialsUpdateArgs),
    /// Delete a material
    Delete(ClassroomMaterialsDeleteArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomMaterialsListArgs {
    /// Course ID
    pub course_id: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomMaterialsGetArgs {
    /// Course ID
    pub course_id: String,
    /// Material ID
    pub material_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomMaterialsCreateArgs {
    /// Course ID
    pub course_id: String,
    /// Title
    #[arg(long)]
    pub title: String,
    /// Description
    #[arg(long)]
    pub description: Option<String>,
    /// Topic ID
    #[arg(long)]
    pub topic_id: Option<String>,
    /// State (PUBLISHED, DRAFT)
    #[arg(long)]
    pub state: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomMaterialsUpdateArgs {
    /// Course ID
    pub course_id: String,
    /// Material ID
    pub material_id: String,
    /// New title
    #[arg(long)]
    pub title: Option<String>,
    /// New description
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomMaterialsDeleteArgs {
    /// Course ID
    pub course_id: String,
    /// Material ID
    pub material_id: String,
}

// ---------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomSubmissionsArgs {
    #[command(subcommand)]
    pub command: ClassroomSubmissionsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomSubmissionsCommand {
    /// List submissions
    List(ClassroomSubmissionsListArgs),
    /// Get submission details
    Get(ClassroomSubmissionsGetArgs),
    /// Turn in a submission
    TurnIn(ClassroomSubmissionsTurnInArgs),
    /// Reclaim a submission
    Reclaim(ClassroomSubmissionsReclaimArgs),
    /// Return a submission
    Return(ClassroomSubmissionsReturnArgs),
    /// Grade a submission
    Grade(ClassroomSubmissionsGradeArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomSubmissionsListArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomSubmissionsGetArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
    /// Submission ID
    pub submission_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomSubmissionsTurnInArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
    /// Submission ID
    pub submission_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomSubmissionsReclaimArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
    /// Submission ID
    pub submission_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomSubmissionsReturnArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
    /// Submission ID
    pub submission_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomSubmissionsGradeArgs {
    /// Course ID
    pub course_id: String,
    /// Coursework ID
    pub coursework_id: String,
    /// Submission ID
    pub submission_id: String,
    /// Assigned grade
    #[arg(long)]
    pub grade: Option<f64>,
    /// Draft grade
    #[arg(long)]
    pub draft_grade: Option<f64>,
}

// ---------------------------------------------------------------
// Announcements
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomAnnouncementsArgs {
    #[command(subcommand)]
    pub command: ClassroomAnnouncementsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomAnnouncementsCommand {
    /// List announcements
    List(ClassroomAnnouncementsListArgs),
    /// Get announcement details
    Get(ClassroomAnnouncementsGetArgs),
    /// Create an announcement
    Create(ClassroomAnnouncementsCreateArgs),
    /// Update an announcement
    Update(ClassroomAnnouncementsUpdateArgs),
    /// Delete an announcement
    Delete(ClassroomAnnouncementsDeleteArgs),
    /// Modify assignees
    Assignees(ClassroomAnnouncementsAssigneesArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomAnnouncementsListArgs {
    /// Course ID
    pub course_id: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomAnnouncementsGetArgs {
    /// Course ID
    pub course_id: String,
    /// Announcement ID
    pub announcement_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomAnnouncementsCreateArgs {
    /// Course ID
    pub course_id: String,
    /// Announcement text
    #[arg(long)]
    pub text: String,
    /// State (PUBLISHED, DRAFT)
    #[arg(long)]
    pub state: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomAnnouncementsUpdateArgs {
    /// Course ID
    pub course_id: String,
    /// Announcement ID
    pub announcement_id: String,
    /// New text
    #[arg(long)]
    pub text: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomAnnouncementsDeleteArgs {
    /// Course ID
    pub course_id: String,
    /// Announcement ID
    pub announcement_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomAnnouncementsAssigneesArgs {
    /// Course ID
    pub course_id: String,
    /// Announcement ID
    pub announcement_id: String,
    /// Student IDs
    #[arg(long)]
    pub add: Vec<String>,
}

// ---------------------------------------------------------------
// Topics
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomTopicsArgs {
    #[command(subcommand)]
    pub command: ClassroomTopicsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomTopicsCommand {
    /// List topics
    List(ClassroomTopicsListArgs),
    /// Get topic details
    Get(ClassroomTopicsGetArgs),
    /// Create a topic
    Create(ClassroomTopicsCreateArgs),
    /// Update a topic
    Update(ClassroomTopicsUpdateArgs),
    /// Delete a topic
    Delete(ClassroomTopicsDeleteArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomTopicsListArgs {
    /// Course ID
    pub course_id: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomTopicsGetArgs {
    /// Course ID
    pub course_id: String,
    /// Topic ID
    pub topic_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomTopicsCreateArgs {
    /// Course ID
    pub course_id: String,
    /// Topic name
    #[arg(long)]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct ClassroomTopicsUpdateArgs {
    /// Course ID
    pub course_id: String,
    /// Topic ID
    pub topic_id: String,
    /// New name
    #[arg(long)]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct ClassroomTopicsDeleteArgs {
    /// Course ID
    pub course_id: String,
    /// Topic ID
    pub topic_id: String,
}

// ---------------------------------------------------------------
// Invitations
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomInvitationsArgs {
    #[command(subcommand)]
    pub command: ClassroomInvitationsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomInvitationsCommand {
    /// List invitations
    List(ClassroomInvitationsListArgs),
    /// Get invitation details
    Get(ClassroomInvitationsGetArgs),
    /// Create an invitation
    Create(ClassroomInvitationsCreateArgs),
    /// Accept an invitation
    Accept(ClassroomInvitationsAcceptArgs),
    /// Delete an invitation
    Delete(ClassroomInvitationsDeleteArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomInvitationsListArgs {
    /// Course ID filter
    #[arg(long)]
    pub course_id: Option<String>,
    /// User ID filter
    #[arg(long)]
    pub user_id: Option<String>,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomInvitationsGetArgs {
    /// Invitation ID
    pub invitation_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomInvitationsCreateArgs {
    /// User ID or email
    #[arg(long)]
    pub user_id: String,
    /// Course ID
    #[arg(long)]
    pub course_id: String,
    /// Role (STUDENT, TEACHER, OWNER)
    #[arg(long)]
    pub role: String,
}

#[derive(Args, Debug)]
pub struct ClassroomInvitationsAcceptArgs {
    /// Invitation ID
    pub invitation_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomInvitationsDeleteArgs {
    /// Invitation ID
    pub invitation_id: String,
}

// ---------------------------------------------------------------
// Guardians
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomGuardiansArgs {
    #[command(subcommand)]
    pub command: ClassroomGuardiansCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomGuardiansCommand {
    /// List guardians for a student
    List(ClassroomGuardiansListArgs),
    /// Get guardian details
    Get(ClassroomGuardiansGetArgs),
    /// Delete a guardian
    Delete(ClassroomGuardiansDeleteArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomGuardiansListArgs {
    /// Student ID
    pub student_id: String,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomGuardiansGetArgs {
    /// Student ID
    pub student_id: String,
    /// Guardian ID
    pub guardian_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomGuardiansDeleteArgs {
    /// Student ID
    pub student_id: String,
    /// Guardian ID
    pub guardian_id: String,
}

// ---------------------------------------------------------------
// Guardian Invitations
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomGuardianInvitationsArgs {
    #[command(subcommand)]
    pub command: ClassroomGuardianInvitationsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ClassroomGuardianInvitationsCommand {
    /// List guardian invitations
    List(ClassroomGuardianInvitationsListArgs),
    /// Get guardian invitation details
    Get(ClassroomGuardianInvitationsGetArgs),
    /// Create a guardian invitation
    Create(ClassroomGuardianInvitationsCreateArgs),
}

#[derive(Args, Debug)]
pub struct ClassroomGuardianInvitationsListArgs {
    /// Student ID
    pub student_id: String,
    /// Filter by state (PENDING, COMPLETE)
    #[arg(long)]
    pub state: Option<String>,
    /// Max results
    #[arg(long, short = 'm')]
    pub max: Option<u32>,
    /// Page token
    #[arg(long)]
    pub page: Option<String>,
}

#[derive(Args, Debug)]
pub struct ClassroomGuardianInvitationsGetArgs {
    /// Student ID
    pub student_id: String,
    /// Invitation ID
    pub invitation_id: String,
}

#[derive(Args, Debug)]
pub struct ClassroomGuardianInvitationsCreateArgs {
    /// Student ID
    pub student_id: String,
    /// Guardian email
    #[arg(long)]
    pub email: String,
}

// ---------------------------------------------------------------
// Profile
// ---------------------------------------------------------------

#[derive(Args, Debug)]
pub struct ClassroomProfileArgs {
    /// User ID (default: "me")
    #[arg(default_value = "me")]
    pub user_id: String,
}
