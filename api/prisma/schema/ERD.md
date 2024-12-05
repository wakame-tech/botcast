```mermaid
erDiagram

        task_status {
            PENDING PENDING
RUNNING RUNNING
COMPLETED COMPLETED
FAILED FAILED
        }
    
  "comments" {
    String id "🗝️"
    String content 
    String user_id 
    String episode_id 
    DateTime created_at 
    }
  

  "episodes" {
    String id "🗝️"
    String title 
    String audio_url "❓"
    Json sections 
    String srt_url "❓"
    String podcast_id 
    String user_id "❓"
    DateTime created_at 
    }
  

  "podcasts" {
    String id "🗝️"
    String title 
    String icon 
    String user_id "❓"
    DateTime created_at 
    }
  

  "scripts" {
    String id "🗝️"
    String title 
    Json template 
    String user_id 
    }
  

  "tasks" {
    String id "🗝️"
    TaskStatus status 
    Json args 
    String user_id "❓"
    DateTime executed_at "❓"
    DateTime executed_finished_at "❓"
    Json result "❓"
    String cron "❓"
    DateTime execute_after 
    }
  

  "users" {
    String id "🗝️"
    String auth_id 
    String email 
    String name "❓"
    }
  
    "comments" o|--|| "users" : "user"
    "comments" o|--|| "episodes" : "episode"
    "episodes" o|--|| "podcasts" : "podcast"
    "episodes" o|--|o "users" : "user"
    "episodes" o{--}o "comments" : "comments"
    "podcasts" o{--}o "episodes" : "episodes"
    "podcasts" o|--|o "users" : "user"
    "scripts" o|--|| "users" : "user"
    "tasks" o|--|| "TaskStatus" : "enum:status"
    "tasks" o|--|o "users" : "user"
    "users" o{--}o "scripts" : "scripts"
    "users" o{--}o "podcasts" : "podcasts"
    "users" o{--}o "episodes" : "episodes"
    "users" o{--}o "tasks" : "tasks"
    "users" o{--}o "comments" : "comments"
```
