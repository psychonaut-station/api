
CREATE DATABASE IF NOT EXISTS apidatabase;
USE apidatabase;

--
-- Table structure for table `ignored_ckeys_autocomplete`
--
DROP TABLE IF EXISTS `ignored_ckeys_autocomplete`;
CREATE TABLE `ignored_ckeys_autocomplete` (
	`id` int(11) NOT NULL AUTO_INCREMENT,
	`ckey` VARCHAR(32) NOT NULL,
	`timestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `valid` BOOLEAN NOT NULL DEFAULT FALSE,
	PRIMARY KEY (`id`)
) ENGINE=InnoDB;