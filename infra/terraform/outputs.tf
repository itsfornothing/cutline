output "eks_cluster_id" {
  description = "EKS Cluster ID"
  value       = module.eks.cluster_id
}

output "eks_cluster_endpoint" {
  description = "Endpoint for EKS control plane"
  value       = module.eks.cluster_endpoint
}

output "render_bucket_name" {
  description = "S3 bucket for rendered videos and assets"
  value       = aws_s3_bucket.renders.id
}

output "render_bucket_arn" {
  description = "ARN of S3 bucket for rendered videos and assets"
  value       = aws_s3_bucket.renders.arn
}
