DROP TABLE IF EXISTS `t_example`;
CREATE TABLE `t_example` (
  `KEY1_id` INT(11) NOT NULL,
  `KEY2_id` INT(11) NOT NULL,
  `field1` INT(11),
  `field2` INT(11),
  `field3` text,
  PRIMARY KEY(`KEY1_id`,`KEY2_id`)
);
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(1,1,123,1,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(2,2,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(3,3,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(4,4,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(5,5,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(6,6,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(7,7,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(8,8,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(9,9,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(10,10,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(11,11,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(12,12,0,2,"hello");
INSERT INTO `t_example`(`KEY1_id`,`KEY2_id`,`field1`,`field2`,`field3`)  VALUES(13,13,0,2,"hello");
