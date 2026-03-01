# Command Reference: omega-google

## Global Flags

| Flag | Short | Env Var | Default | Description |
|------|-------|---------|---------|-------------|
| `--json` | `-j` | `GOG_JSON` | false | Output JSON to stdout |
| `--plain` | `-p` | `GOG_PLAIN` | false | Output stable TSV to stdout |
| `--color` | | `GOG_COLOR` | auto | Color mode: auto/always/never |
| `--account` | `-a` | `GOG_ACCOUNT` | (resolved) | Google account email or alias |
| `--client` | | `GOG_CLIENT` | default | OAuth client name |
| `--verbose` | `-v` | | false | Enable verbose logging |
| `--dry-run` | `-n` | | false | Preview without making changes |
| `--force` | `-y` | | false | Skip confirmation prompts |
| `--no-input` | | | false | Never prompt; fail instead |
| `--results-only` | | | false | Strip envelope from JSON output |
| `--select` | | | | Filter JSON fields (comma-separated) |
| `--enable-commands` | | `GOG_ENABLE_COMMANDS` | | Restrict allowed commands |
| `--version` | | | | Print version and exit |

## Command Tree

### Auth Commands (M1)

```
omega-google auth credentials <file|->       Store OAuth client credentials
omega-google auth credentials list            List stored credential files
omega-google auth add <email>                 OAuth2 flow to store refresh token
    --services <csv>                          Services to authorize (default: user services)
    --readonly                                Request readonly scope variants
    --drive-scope full|readonly|file          Drive scope mode
    --manual                                  Browserless flow (paste URL)
    --remote --step 1|2                       Remote two-step flow
    --auth-url <url>                          URL for --remote --step 2
    --force-consent                           Force consent prompt
    --timeout <duration>                      Auth flow timeout
omega-google auth remove <email>              Remove stored refresh token
omega-google auth list                        List all stored accounts
omega-google auth status                      Show auth configuration summary
omega-google auth services                    List services with OAuth scopes
    --markdown                                Output as markdown table
omega-google auth tokens list                 List keyring token keys
omega-google auth tokens delete <email>       Delete token from keyring
omega-google auth alias set <alias> <email>   Set account alias
omega-google auth alias unset <alias>         Remove account alias
omega-google auth alias list                  List all aliases
omega-google auth keep <email> --key <sa.json>  Store service account for Keep
omega-google auth keyring                     Show/change keyring backend
```

### Config Commands (M1)

```
omega-google config get <key>                 Get config value
omega-google config set <key> <value>         Set config value
omega-google config unset <key>               Remove config key
omega-google config list                      List all config key-value pairs
omega-google config keys                      List valid config key names
omega-google config path                      Print config file path
```

### Utility Commands (M1)

```
omega-google version                          Print version info
omega-google time now                         Show current time
    --timezone <tz>                           IANA timezone name
```

### Gmail Commands (M2)

```
omega-google gmail search <query>             Search threads
    --max N --page TOKEN --all --fail-empty --oldest --timezone TZ
omega-google gmail messages search <query>    Search messages
    --include-body --max N --page TOKEN
omega-google gmail thread get <threadId>      Get thread with messages
    --download
omega-google gmail thread modify <threadId>   Add/remove labels on thread
    --add LABEL --remove LABEL
omega-google gmail thread attachments <id>    Download thread attachments
    --out-dir DIR
omega-google gmail get <messageId>            Get single message
    --format full|metadata|raw --headers HEADER,...
omega-google gmail attachment <msgId> <attId> Download attachment
    --out PATH --name NAME
omega-google gmail url <threadIds...>         Print Gmail web URLs
omega-google gmail labels list                List all labels
omega-google gmail labels get <labelIdOrName> Get label details
omega-google gmail labels create <name>       Create label
omega-google gmail labels modify <ids...>     Batch modify labels
    --add LABEL --remove LABEL
omega-google gmail labels delete <labelId>    Delete label
omega-google gmail send                       Send email
    --to ADDR --subject S --body B --body-html H
    --cc ADDR --bcc ADDR --reply-to-message-id ID
    --reply-to ADDR --attach FILE... --track
omega-google gmail drafts list                List drafts
omega-google gmail drafts get <draftId>       Get draft
omega-google gmail drafts create              Create draft (same flags as send)
omega-google gmail drafts update <draftId>    Update draft
omega-google gmail drafts send <draftId>      Send draft
omega-google gmail drafts delete <draftId>    Delete draft
omega-google gmail watch start|status|renew|stop|serve
omega-google gmail history --since <historyId>
omega-google gmail batch delete <ids...>      Batch delete messages
omega-google gmail batch modify <ids...>      Batch modify labels
omega-google gmail settings filters list|get|create|delete
omega-google gmail settings forwarding list|get|create|delete
omega-google gmail settings sendas list|get|create|verify|delete|update
omega-google gmail settings delegates list|get|add|remove
omega-google gmail settings vacation get|update
omega-google gmail settings autoforward get|update
```

### Calendar Commands (M2)

```
omega-google calendar calendars               List calendars
omega-google calendar acl <calendarId>        List ACL entries
omega-google calendar events                  List events
    --cal ID_OR_NAME --calendars CSV --all
    --from DT --to DT --max N --page TOKEN --query Q
    --weekday --fields FIELDS
omega-google calendar event <calId> <eventId> Get single event
omega-google calendar create                  Create event
    --summary S --from DT --to DT --description D
    --location L --attendees CSV --all-day --event-type TYPE
omega-google calendar update <calId> <eventId>
    (same flags as create, all optional)
    --add-attendee ADDR
omega-google calendar delete <calId> <eventId>
omega-google calendar freebusy <calIds> --from DT --to DT
omega-google calendar respond <calId> <eventId>
    --status accepted|declined|tentative
    --send-updates all|none|externalOnly
omega-google calendar search <query>
omega-google calendar time
omega-google calendar users
omega-google calendar team <groupEmail> --from DT --to DT
omega-google calendar colors
omega-google calendar conflicts --from DT --to DT
omega-google calendar propose-time <calId> <eventId>
omega-google calendar focus-time --from DT --to DT
omega-google calendar out-of-office --from DT --to DT
omega-google calendar working-location --from DT --to DT
```

### Drive Commands (M2)

```
omega-google drive ls                         List files
    --parent ID --max N --page TOKEN --query Q --[no-]all-drives
omega-google drive search <text>              Search files
    --raw-query --max N --page TOKEN --[no-]all-drives
omega-google drive get <fileId>               Get file metadata
omega-google drive download <fileId>          Download file
    --out PATH --format FORMAT
omega-google drive upload <localPath>         Upload file
    --name N --parent ID --convert --convert-to TYPE
omega-google drive mkdir <name>               Create folder
    --parent ID
omega-google drive delete <fileId>            Delete file
    --permanent
omega-google drive move <fileId> --parent ID  Move file
omega-google drive rename <fileId> <newName>  Rename file
omega-google drive share <fileId>             Share file
    --to anyone|user|domain --email ADDR --domain D
    --role reader|writer --discoverable
omega-google drive permissions <fileId>       List permissions
omega-google drive unshare <fileId> <permId>  Remove permission
omega-google drive url <fileIds...>           Print web URLs
omega-google drive drives                     List shared drives
omega-google drive copy <fileId>              Copy file
    --name N --parent ID
omega-google drive comments list|get|create|update|delete|reply
```

### Docs Commands (M3)

```
omega-google docs export <docId>              Export document
    --format pdf|docx|txt --out PATH
omega-google docs info <docId>                Get metadata
omega-google docs create <title>              Create document
    --parent ID --file PATH
omega-google docs copy <docId> <title>        Copy document
    --parent ID
omega-google docs cat <docId>                 Extract plain text
    --max-bytes N --tab TAB --all-tabs --raw
omega-google docs list-tabs <docId>           List document tabs
omega-google docs comments list|get|add|reply|resolve|delete
omega-google docs write <docId> [content]     Append/replace content
    --file PATH --replace --markdown
omega-google docs insert <docId> [content]    Insert at position
    --index N --file PATH
omega-google docs delete <docId>              Delete text range
    --start N --end N
omega-google docs find-replace <docId> <find> <replace>
    --match-case
omega-google docs edit <docId>                Find/replace with flags
    --find PATTERN --replace TEXT --match-case
omega-google docs update <docId>              Update with format
    --content TEXT --content-file PATH --format plain|markdown --append
omega-google docs sed <docId> <expression>    Sed-like editing
    -e EXPR -f FILE
omega-google docs clear <docId>               Clear all content
```

### Sheets Commands (M3)

```
omega-google sheets get <id> <range>          Read cells
    --dimension ROWS|COLUMNS --render FORMAT
omega-google sheets update <id> <range>       Write cells
    --values-json JSON --input RAW|USER_ENTERED
    --copy-validation-from RANGE
omega-google sheets append <id> <range>       Append rows
    --insert OVERWRITE|INSERT_ROWS --copy-validation-from RANGE
omega-google sheets insert <id> <sheet> <dim> <start>
    --count N --after
omega-google sheets clear <id> <range>        Clear values
omega-google sheets format <id> <range>       Apply formatting
    --format-json JSON --format-fields MASK
omega-google sheets notes <id> <range>        Read cell notes
omega-google sheets metadata <id>             Spreadsheet metadata
omega-google sheets create <title>            Create spreadsheet
    --sheets NAME,...
omega-google sheets copy <id> <title>         Copy spreadsheet
    --parent ID
omega-google sheets export <id>               Export
    --format pdf|xlsx|csv --out PATH
```

### Slides Commands (M3)

```
omega-google slides export <id>               Export presentation
    --format pdf|pptx --out PATH
omega-google slides info <id>                 Get metadata
omega-google slides create <title>            Create presentation
    --parent ID --template ID
omega-google slides create-from-markdown      From markdown
    --content TEXT --content-file PATH --parent ID
omega-google slides copy <id> <title>         Copy presentation
    --parent ID
omega-google slides list-slides <id>          List slides
omega-google slides add-slide <id>            Add slide
omega-google slides delete-slide <id> <slideId>
omega-google slides read-slide <id> <slideId>
omega-google slides update-notes <id> <slideId>
omega-google slides replace-slide <id> <slideId>
```

### Forms Commands (M3)

```
omega-google forms get <formId>               Get form metadata
omega-google forms create --title TITLE       Create form
    --description TEXT
omega-google forms responses list <formId>    List responses
    --max N --page TOKEN --filter FILTER
omega-google forms responses get <formId> <responseId>
```

### Chat Commands (M4)

```
omega-google chat spaces list                 List spaces
omega-google chat spaces find <displayName>   Find space
omega-google chat spaces create <displayName> Create space
    --member EMAIL,...
omega-google chat messages list <space>       List messages
    --max N --page TOKEN --order ORDER --thread ID --unread
omega-google chat messages send <space>       Send message
    --text TEXT --thread ID
omega-google chat threads list <space>        List threads
omega-google chat dm space <email>            Get DM space
omega-google chat dm send <email>             Send DM
    --text TEXT --thread ID
```

### Classroom Commands (M4)

```
omega-google classroom courses list|get|create|update|delete|archive|unarchive|join|leave|url
omega-google classroom students list|get|add|remove
omega-google classroom teachers list|get|add|remove
omega-google classroom roster <courseId>
omega-google classroom coursework list|get|create|update|delete|assignees
omega-google classroom materials list|get|create|update|delete
omega-google classroom submissions list|get|turn-in|reclaim|return|grade
omega-google classroom announcements list|get|create|update|delete|assignees
omega-google classroom topics list|get|create|update|delete
omega-google classroom invitations list|get|create|accept|delete
omega-google classroom guardians list|get|delete
omega-google classroom guardian-invitations list|get|create
omega-google classroom profile [userId]
```

### Tasks Commands (M4)

```
omega-google tasks lists                      List task lists
omega-google tasks lists create <title>       Create task list
omega-google tasks list <tasklistId>          List tasks
omega-google tasks get <tasklistId> <taskId>  Get task
omega-google tasks add <tasklistId>           Add task
    --title T --notes N --due DATE
    --repeat daily|weekly|monthly|yearly
    --repeat-count N --repeat-until DT
    --parent ID --previous ID
omega-google tasks update <tasklistId> <taskId>
    --title T --notes N --due DATE --status needsAction|completed
omega-google tasks done <tasklistId> <taskId>
omega-google tasks undo <tasklistId> <taskId>
omega-google tasks delete <tasklistId> <taskId>
omega-google tasks clear <tasklistId>
```

### Contacts Commands (M4)

```
omega-google contacts search <query>          Search contacts
omega-google contacts list                    List all contacts
omega-google contacts get <resourceName|email>
omega-google contacts create                  Create contact
    --given NAME --family NAME --email ADDR --phone NUM
omega-google contacts update <resourceName>   Update contact
    --given NAME --family NAME --email ADDR --phone NUM
    --birthday DATE --notes TEXT --from-file PATH|- --ignore-etag
omega-google contacts delete <resourceName>
omega-google contacts directory list|search
omega-google contacts other list|search
```

### People Commands (M4)

```
omega-google people me                        Show your profile
omega-google people get <resourceName|userId>
omega-google people search <query>
omega-google people relations [resourceName]  Show relations
    --type TYPE
```

### Groups Commands (M5)

```
omega-google groups list                      List groups
omega-google groups members <groupEmail>      List members
```

### Keep Commands (M5)

```
omega-google keep list                        List notes
    --max N --page TOKEN --all --filter EXPR
omega-google keep get <noteId>                Get note
omega-google keep search <query>              Search notes
omega-google keep attachment <name>           Download attachment
    --mime-type TYPE --out PATH
```

### Apps Script Commands (M5)

```
omega-google appscript get <scriptId>         Get project
omega-google appscript content <scriptId>     Get source files
omega-google appscript run <scriptId> <func>  Run function
    --params JSON --dev-mode
omega-google appscript create --title TITLE   Create project
    --parent-id ID
```

### Desire Path Aliases (M2)

| Alias | Equivalent | Description |
|-------|-----------|-------------|
| `send` | `gmail send` | Send an email |
| `ls` / `list` | `drive ls` | List Drive files |
| `search` / `find` | `drive search` | Search Drive files |
| `download` / `dl` | `drive download` | Download a file |
| `upload` / `up` | `drive upload` | Upload a file |
| `login` | `auth add` | Authorize an account |
| `logout` | `auth remove` | Remove an account |
| `status` / `st` | `auth status` | Show auth status |
| `me` | `people me` | Show your profile |
| `whoami` | `people me` | Show your profile |
| `open` / `browse` | `open` | Open URL for Google ID |

### Agent/Machine Commands (M6)

```
omega-google agent exit-codes                 Print exit code table
omega-google schema [command]                 Machine-readable CLI schema
    --include-hidden
omega-google exit-codes                       Alias for agent exit-codes
omega-google completion <shell>               Generate shell completions
    bash|zsh|fish|powershell
```

## Exit Codes

| Code | Name | Meaning |
|------|------|---------|
| 0 | Success | Command completed successfully |
| 1 | Error | Generic error |
| 2 | Usage | Invalid arguments or parse error |
| 3 | Empty | No results (with `--fail-empty`) |
| 4 | AuthRequired | Authentication needed |
| 5 | NotFound | Resource not found |
| 6 | PermissionDenied | Insufficient permissions |
| 7 | RateLimited | API rate limit exceeded |
| 8 | Retryable | Transient error (circuit breaker) |
| 10 | ConfigError | Configuration problem |
| 130 | Cancelled | Interrupted (SIGINT) |
