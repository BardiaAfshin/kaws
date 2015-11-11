# Clusters

A kaws cluster is a Kubernetes cluster managed by kaws.
Clusters are isolated from each other because they each exist in their own [AWS VPC](https://aws.amazon.com/vpc/).
The AWS resources that comprise a cluster are defined in kaws's Terraform module, which is declared in the file `terraform/main.tf` of a [kaws repository](repository.md).

Each Kubernetes cluster created by kaws:

* Uses CoreOS as the operating system for each server
* Has one bastion server that allows external SSH access
* Has three servers dedicated to running [etcd](https://coreos.com/etcd/)
* Bootstraps etcd statically so no discovery token is required
* Has two Kubernetes master servers backed by an [AWS ELB](https://aws.amazon.com/elasticloadbalancing/)
* Uses master election of the Kubernetes master servers for high availability
* Has two Kubernetes node servers
* Uses SSL client certificates for authentication to the Kubernetes API
* Accepts external traffic to the Kubernetes API only via SSL on port 443
* Accepts external traffic to Kubernetes nodes only on port 80 and 443 (though you should use HSTS to redirect requests from 80 to 443)
* Includes Kubernetes's DNS addon, making all Kubernetes services all discoverable via DNS
* Has a DNS record for the Kubernetes API at kubernetes.example.com, where example.com is a value set at cluster creation time
* Has a DNS record for the bastion SSH server at bastion.example.com, where example.com is a value set at cluster creation time
