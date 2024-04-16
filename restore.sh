#!/bin/bash

# Set start and end date
START_DATE="2024-04-01"  # YYYY-MM-DD
END_DATE="2024-11-30"    # YYYY-MM-DD

# Min and max commits per selected day
MIN_COMMITS=0
MAX_COMMITS=2

# Min and max days per month to commit
MIN_DAYS_PER_MONTH=5
MAX_DAYS_PER_MONTH=20

# Set author and committer name and email
AUTHOR_NAME="darkhorse-techsolution"
AUTHOR_EMAIL="paulovitor.medeiros@hotmail.com"

# Define an array of commit messages (20 total)
commit_messages=(
    "Feat: add port"
    "remove: web hook"
    "Feat: add settings for websocket"
    "Feat: add wallet header and wallet address and health check function"
    "Fix: display movie"
    "Feat: add movie"
    "Fix: update readme"
    "Fix: update readme"
    "Fix: update readme => add video"
    "Feat: add helper video"
    "Fix: update package.json"
    "Fix: amount issue for v2"
    "Fix: amount issue"
    "Fix: fix amount issue"
    "Fix: fix insufficient PLS issue"
    "Update: update readme"
    "Feat: update readme.md"
    "Fix: fix all issue"
    "Fix: queued order issue"
    "Feat: add failed status also"
    "Feat: add queued transaction list"
    "Feat: add queued transcation list"
    "Feat: add retrying when snipe"
    "Fix: update hex value"
    "Feat: add approving preprocess"
    "Feat: add pending queue, fix swap error"
    "Remove: videos"
    "Feat: add approve"
    "Feat: add snipe custom amount to swap"
    "Fix hex issue"
    "add custom amount"
    "Fix: update swap amount"
    "Feat: add multiple swap"
    "Feat: update direct swap label"
    "Feat: add direct swap for UI"
    "Remove: rpc link on env"
    "Feat: add rpc"
    "Fix: fix alert issue"
    "Feat: add snipe function"
    "Fix: fix target token and token0, token1"
    "Feat: add direct swap"
    "Feat: update snipe"
    "fix ui"
    "Fix: fix swap issue"
    "Fix: swap issue"
    "Feat: update UI for responsibility"
    "Update setting.json"
    "Feat: add snipe logic"
    "Fix: pair addres issue"
    "Feat: add swap logic"
    "Fix: fix socket issue"
    "Feat: update server"
    "Fix: update ui again"
    "Feat: improve ui with responsive"
    "Feat: update ui for snipe pool"
    "Feat: add token list ui and send new pair"
    "Feat: add readme.md"
    "Feat: add snipe ui, implement websocket between ui and bot"
    "Feat: add ui"
    "Feat: add settings"
    "Feat: get liquidity"
    "Feat: add version for pool"
    "Feat: get pool launch"
    "Feat: add log monitor"
    "Delete .idea directory"
    "Fix: remove .idea from git history"
    "Feat: add web server"
)




# Convert START_DATE to first day of the month
current_date=$(date -d "$START_DATE" +"%Y-%m-01")

# Loop through each month in the date range
while [[ "$current_date" < "$END_DATE" ]] || [[ "$current_date" == "$END_DATE" ]]; do
    # Get number of days in the current month
    days_in_month=$(date -d "$current_date +1 month -1 day" +"%d")

    # Random number of commit days for the month
    num_commit_days=$(( RANDOM % (MAX_DAYS_PER_MONTH - MIN_DAYS_PER_MONTH + 1) + MIN_DAYS_PER_MONTH ))

    # Select random days in the month
    commit_days=()
    while [[ ${#commit_days[@]} -lt $num_commit_days ]]; do
        random_day=$(( RANDOM % days_in_month + 1 ))
        if [[ ! " ${commit_days[@]} " =~ " $random_day " ]]; then
            commit_days+=("$random_day")
        fi
    done

    # Make commits on selected random days
    for day in "${commit_days[@]}"; do
        commit_date=$(date -d "$current_date +$((day-1)) days" +"%Y-%m-%d")

        # Random number of commits for the selected day
        num_commits=$(( RANDOM % (MAX_COMMITS - MIN_COMMITS + 1) + MIN_COMMITS ))

        for ((i=1; i<=num_commits; i++)); do
            # Generate a random timestamp within the day
            commit_time=$(date -d "$commit_date $((RANDOM % 24)):$((RANDOM % 60)):$((RANDOM % 60))" +"%Y-%m-%d %H:%M:%S")

            # Pick a random commit message from the array
            commit_message=${commit_messages[$RANDOM % ${#commit_messages[@]}]}

            # Create or modify a file
            echo "Commit on $commit_time" >> dummy.txt

            # Add changes
            git add .

            # Commit with specific author details and timestamp
            GIT_AUTHOR_NAME="$AUTHOR_NAME" GIT_AUTHOR_EMAIL="$AUTHOR_EMAIL" \
            GIT_COMMITTER_NAME="$AUTHOR_NAME" GIT_COMMITTER_EMAIL="$AUTHOR_EMAIL" \
            GIT_COMMITTER_DATE="$commit_time" GIT_AUTHOR_DATE="$commit_time" \
            git commit -m "$commit_message"
        done
    done

    # Move to the first day of the next month
    current_date=$(date -d "$current_date +1 month" +"%Y-%m-01")
done