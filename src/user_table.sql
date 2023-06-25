CREATE TABLE IF NOT EXISTS users (
  Id int NOT NULL AUTO_INCREMENT,
  Username VARCHAR(255) NOT NULL UNIQUE,
  Password CHAR(64) NOT NULL, 
  PRIMARY KEY (Id)
);
