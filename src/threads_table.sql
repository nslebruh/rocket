CREATE TABLE IF NOT EXISTS threads (
  id int NOT NULL AUTO_INCREMENT,
  userId int NOT NULL,
  floss int NOT NULL,
  amount int NOT NULL,
  name VARCHAR(255) NOT NULL,
  color VARCHAR(512) NOT NULL, 
  PRIMARY KEY (Id)
)
