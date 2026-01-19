/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `hid_ckeys_autocomplete`
--

DROP TABLE IF EXISTS `hid_ckeys_autocomplete`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `hid_ckeys_autocomplete` (
	`id` INT(11) NOT NULL AUTO_INCREMENT,
	`ckey` VARCHAR(32) NOT NULL COLLATE 'utf8mb4_general_ci',
	`hid_by` BIGINT(20) NOT NULL,
	`unhid_by` BIGINT(20) NULL DEFAULT NULL,
	`timestamp` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	`valid` BOOLEAN NOT NULL DEFAULT FALSE,
	PRIMARY KEY (`id`)
) COLLATE='utf8mb4_general_ci' ENGINE=InnoDB;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `friendship`
--
DROP TABLE IF EXISTS `friendship`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `friendship` (
  `id` INT NOT NULL AUTO_INCREMENT,
  `user_ckey` VARCHAR(32) NOT NULL,
  `friend_ckey` VARCHAR(32) NOT NULL,
  `status` enum('pending','accepted','declined','removed') NOT NULL DEFAULT 'pending',
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `unique_pair` VARCHAR(65) AS (
    IF(user_ckey < friend_ckey, 
       CONCAT(user_ckey, ':', friend_ckey), 
       CONCAT(friend_ckey, ':', user_ckey))
  ) VIRTUAL,
  PRIMARY KEY (`id`),
  UNIQUE INDEX `unique_constraints` (`unique_pair`),
  CHECK (user_ckey <> friend_ckey)
) COLLATE='utf8mb4_general_ci' ENGINE=InnoDB;
/*!40101 SET character_set_client = @saved_cs_client */;