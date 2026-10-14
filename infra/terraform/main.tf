terraform {
  required_version = ">= 1.5.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# 1. Output Asset & Artifact S3 Bucket
resource "aws_s3_bucket" "renders" {
  bucket        = "cutline-renders-${var.environment}-${var.aws_region}"
  force_destroy = false

  tags = {
    Environment = var.environment
    Service     = "cutline"
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "renders_lifecycle" {
  bucket = aws_s3_bucket.renders.id

  rule {
    id     = "expire-scratch-and-temp"
    status = "Enabled"

    filter {
      prefix = "temp/"
    }

    expiration {
      days = 7
    }
  }
}

resource "aws_s3_bucket_cors_configuration" "renders_cors" {
  bucket = aws_s3_bucket.renders.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "PUT", "POST", "HEAD"]
    allowed_origins = ["*"]
    expose_headers  = ["ETag"]
    max_age_seconds = 3600
  }
}

# 2. VPC & Networking for Low-Latency GPU Cluster
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = "cutline-vpc-${var.environment}"
  cidr = "10.0.0.0/16"

  azs             = ["${var.aws_region}a", "${var.aws_region}b", "${var.aws_region}c"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]

  enable_nat_gateway     = true
  single_nat_gateway     = var.environment != "production"
  enable_dns_hostnames   = true
  enable_dns_support     = true

  tags = {
    Environment = var.environment
    Service     = "cutline"
  }
}

# 3. EKS Cluster
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = var.cluster_name
  cluster_version = "1.30"

  cluster_endpoint_public_access = true
  vpc_id                         = module.vpc.vpc_id
  subnet_ids                     = module.vpc.private_subnets

  eks_managed_node_groups = {
    # System / API node group (CPU nodes for Axum API, scheduler, monitoring)
    api_pool = {
      name           = "cutline-api-pool"
      instance_types = ["c6i.xlarge"]
      min_size       = 2
      max_size       = 10
      desired_size   = 2

      labels = {
        role = "api"
      }
    }

    # GPU Worker pool (NVIDIA T4 / A10G GPU nodes for compositing, forced alignment, NVENC encoding)
    gpu_worker_pool = {
      name           = "cutline-gpu-workers"
      instance_types = var.gpu_instance_types
      min_size       = var.gpu_min_size
      max_size       = var.gpu_max_size
      desired_size   = var.gpu_desired_capacity

      ami_type = "AL2_x86_64_GPU" # Includes NVIDIA CUDA/NVENC drivers out-of-the-box

      labels = {
        role        = "worker"
        accelerator = "nvidia-tesla-t4"
      }

      taints = [
        {
          key    = "nvidia.com/gpu"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      ]

      block_device_mappings = {
        xvda = {
          device_name = "/dev/xvda"
          ebs = {
            volume_size           = 150
            volume_type           = "gp3"
            iops                  = 4000
            throughput            = 250
            delete_on_termination = true
          }
        }
      }
    }
  }

  tags = {
    Environment = var.environment
    Service     = "cutline"
  }
}
