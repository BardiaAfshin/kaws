resource "aws_s3_bucket" "cloud_config" {
  bucket = "kaws-${var.account_id}-${var.cluster}"

  tags {
    Name = "kaws-${var.account_id}-${var.cluster}"
    Cluster = "${var.cluster}"
  }
}

resource "aws_s3_bucket_object" "bastion_cloud_config" {
  bucket = "kaws-${var.account_id}-${var.cluster}"
  key = "bastion_cloud_config.yml"
  content = "${data.template_file.bastion_cloud_config.rendered}"
}

resource "aws_s3_bucket_object" "etcd_01_cloud_config" {
  bucket = "kaws-${var.account_id}-${var.cluster}"
  key = "etcd_01_cloud_config.yml"
  content = "${data.template_file.etcd_01_cloud_config.rendered}"
}

resource "aws_s3_bucket_object" "etcd_02_cloud_config" {
  bucket = "kaws-${var.account_id}-${var.cluster}"
  key = "etcd_02_cloud_config.yml"
  content = "${data.template_file.etcd_02_cloud_config.rendered}"
}

resource "aws_s3_bucket_object" "etcd_03_cloud_config" {
  bucket = "kaws-${var.account_id}-${var.cluster}"
  key = "etcd_03_cloud_config.yml"
  content = "${data.template_file.etcd_03_cloud_config.rendered}"
}

resource "aws_s3_bucket_object" "master_cloud_config" {
  bucket = "kaws-${var.account_id}-${var.cluster}"
  key = "master_cloud_config.yml"
  content = "${data.template_file.master_cloud_config.rendered}"
}

resource "aws_s3_bucket_object" "node_cloud_config" {
  bucket = "kaws-${var.account_id}-${var.cluster}"
  key = "node_cloud_config.yml"
  content = "${data.template_file.node_cloud_config.rendered}"
}
