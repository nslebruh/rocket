# README 

## ThreadTracker


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