CREATE TABLE IF NOT EXISTS threads (
  Id int NOT NULL AUTO_INCREMENT,
  UserId int NOT NULL,
  Floss int NOT NULL,
  Amount int NOT NULL,
  PRIMARY KEY (Id)
)