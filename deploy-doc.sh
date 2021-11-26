cargo doc --target-dir ./target/docs
aws s3 sync ./target/docs/ ${DEPLOY_S3_BUCKET}
aws cloudfront --region ${AWS_REGION} create-invalidation --distribution-id ${CF_DISTRIBUTION_ID} --paths "/*" "/"

echo "S3 upload success!!!, trigger cache invalidation..."
invalidation_id=$(aws cloudfront --region ${AWS_REGION} create-invalidation --distribution-id ${CF_DISTRIBUTION_ID} --query 'Location.Invalidation.Id' --output text --paths "/*" "/")
aws cloudfront --region ${AWS_REGION} wait invalidation-completed --distribution-id ${CF_DISTRIBUTION_ID} --id ${invalidation_id}