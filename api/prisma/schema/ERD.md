```mermaid
erDiagram

        task_status {
            PENDING PENDING
RUNNING RUNNING
COMPLETED COMPLETED
FAILED FAILED
        }
    
  "corners" {
    String id "🗝️"
    String title 
    String description 
    Boolean requesting_mail 
    Json mail_schema 
    String user_id 
    }
  

  "episodes" {
    String id "🗝️"
    String title 
    String audio_url "❓"
    String user_id "❓"
    String podcast_id 
    String srt_url "❓"
    DateTime created_at 
    Json sections 
    String description "❓"
    Int duration_sec "❓"
    }
  

  "mails" {
    String id "🗝️"
    String name 
    Json body 
    String user_id 
    String corner_id 
    DateTime created_at 
    }
  

  "podcasts" {
    String id "🗝️"
    String title 
    String user_id "❓"
    String icon 
    DateTime created_at 
    String description "❓"
    }
  

  "scripts" {
    String id "🗝️"
    Json template 
    String user_id 
    String title 
    String description "❓"
    Json arguments 
    }
  

  "tasks" {
    String id "🗝️"
    TaskStatus status 
    Json args 
    String user_id "❓"
    DateTime execute_after 
    DateTime executed_at "❓"
    DateTime executed_finished_at "❓"
    Json result "❓"
    String cron "❓"
    }
  

  "users" {
    String id "🗝️"
    String auth_id 
    String email 
    String name "❓"
    }
  
    "corners" o|--|| "users" : "user"
    "corners" o{--}o "podcasts" : "podcasts"
    "corners" o{--}o "mails" : "Mail"
    "episodes" o|--|| "podcasts" : "podcast"
    "episodes" o|--|o "users" : "user"
    "mails" o|--|| "users" : "user"
    "mails" o|--|| "corners" : "corner"
    "podcasts" o{--}o "episodes" : "episodes"
    "podcasts" o{--}o "corners" : "corners"
    "podcasts" o|--|o "users" : "user"
    "scripts" o|--|| "users" : "user"
    "tasks" o|--|| "TaskStatus" : "enum:status"
    "tasks" o|--|o "users" : "user"
    "users" o{--}o "episodes" : "episodes"
    "users" o{--}o "podcasts" : "podcasts"
    "users" o{--}o "scripts" : "scripts"
    "users" o{--}o "tasks" : "tasks"
    "users" o{--}o "corners" : "corners"
    "users" o{--}o "mails" : "mails"
```
