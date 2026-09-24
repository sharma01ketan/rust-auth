# Task API

A local task-management API for signing in, assigning work, and seeing only the work assigned to you.

## Language

**User**:
A person who can sign in. The two seed users are Admin and James Bond.
_Avoid_: Account, member

**Role**:
The permission level of a User: admin or staff.
_Avoid_: Admin (when meaning the permission), permission

**Task**:
A unit of work with a title, description, status, and priority, created by an admin.
_Avoid_: Ticket, item, todo

**Assignee**:
The User a Task is assigned to. A Task has at most one Assignee.
_Avoid_: Owner, recipient

**Assignment**:
The link from a Task to its Assignee. Changing it is an admin action.
_Avoid_: Allocation, handoff

**Login challenge**:
A pending sign-in that must be confirmed with a verification code before the User has a session.
_Avoid_: OTP session, 2FA token, MFA challenge

**Verification code**:
The one-time code issued for a login challenge.
_Avoid_: OTP, PIN, token

**Email log**:
The record of a verification code as it was issued, so a local reviewer can read the code.
_Avoid_: Mailbox, outbox, email client

**My tasks**:
The tasks whose Assignee is the signed-in User.
_Avoid_: Inbox, task list
