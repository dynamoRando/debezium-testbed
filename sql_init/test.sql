CREATE DATABASE mydb;
CREATE USER 'testbed'@'localhost' IDENTIFIED BY 'testbed';
GRANT ALL PRIVILEGES ON mydb.* TO 'testbed'@'%' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON mydb.* TO 'testbed'@'localhost' WITH GRANT OPTION;
FLUSH PRIVILEGES;
USE mydb

CREATE TABLE example (
    id INT
);