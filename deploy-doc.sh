cargo doc --target-dir ./target/docs
aws s3 sync ./target/docs/ ${DEPLOY_S3_BUCKET}
aws cloudfront --region ${AWS_REGION} create-invalidation --distribution-id ${CF_DISTRIBUTION_ID} --paths "/*" "/"