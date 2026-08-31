# Database schemas

- The reviews don't have an on delete cascade declaration, making character
  deletion a huge hassle.
- Same applies for the reviewer column, making deletion of old storytellers an
  issue.
