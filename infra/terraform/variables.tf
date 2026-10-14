variable "aws_region" {
  description = "AWS region for Cutline deployment"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name (production, staging)"
  type        = string
  default     = "production"
}

variable "cluster_name" {
  description = "Name of the EKS cluster"
  type        = string
  default     = "cutline-production"
}

variable "gpu_instance_types" {
  description = "EC2 instance types for GPU render worker pool (NVIDIA Tesla T4 or A10G)"
  type        = list(string)
  default     = ["g4dn.xlarge", "g4dn.2xlarge"]
}

variable "gpu_min_size" {
  description = "Minimum number of warm GPU nodes in the pool"
  type        = number
  default     = 2
}

variable "gpu_max_size" {
  description = "Maximum number of GPU worker nodes under peak load"
  type        = number
  default     = 32
}

variable "gpu_desired_capacity" {
  description = "Desired number of warm GPU nodes"
  type        = number
  default     = 4
}
