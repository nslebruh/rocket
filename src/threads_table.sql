CREATE TABLE IF NOT EXISTS threads (
  userId int NOT NULL,
  floss int NOT NULL,
  amount int NOT NULL,
  name VARCHAR(255) NOT NULL,
  color VARCHAR(255) NOT NULL, 
  PRIMARY KEY (userId, floss)
)
