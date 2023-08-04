# README 

## ThreadTracker
Created by Nathan Le Brun
VSV ID: 107027

## Installation Instructions
To run locally:
1. Download and install Docker Desktop from their [website](https://docs.docker.com/desktop/install/windows-install/).
2. Run the command `docker compose up`
3. Go to this local [webpage](http://localhost:8024) 
 
**OR** you can just use the live version [here](https://threads.clompass.com)

### Testing Techniques
- Functional testing
- Test each function to ensure that it works (clickable components, data storage and retrieval)
- Test each data entry for authentication (username and password)
- Test incremental quantity controls (increment and decrement)
- Test that it works on both mobile and desktop

### Validation
Due to the nature of the rust programming language, data 
The username and password is validated with the database

### Data Table
| Data           | Type             | Description / Use |
| -------------- | ---------------- | ----------------- |
| user.username  | string           |                   |
| user.password  | string           |                   |
| user.id        | unsigned integer |                   |
| thread.floss   | unsigned integer |                   |
| thread.amount  | unsigned integer |                   |
| thread.colour  | string           |                   |
| thread.user_id | unsigned integer |                   |
| thread.name    | string           |                   |