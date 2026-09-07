# externals/aws-cloudformation-templates/RainModules/bucket.yml

> block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -6,9 +6,9 @@
   AppName:
     Type: String
     Description: |
       This string will serve as a prefix for all resource names, which have 
-      the general form of AppName-ResourceName-Region-Account.
+      the general form of AppName-ResourceName-Region-Account. 
 
   Content:
     Type: String
     Description: A local path to a directory that will be uploaded to the bucket

`````

### Actual (oxfmt)

`````yml
Description: |
  This module creates an S3 bucket that will pass common compliance checks 
  by default. It also creates an associated log bucket and replica bucket.

Parameters:
  AppName:
    Type: String
    Description: |
      This string will serve as a prefix for all resource names, which have 
      the general form of AppName-ResourceName-Region-Account. 

  Content:
    Type: String
    Description: A local path to a directory that will be uploaded to the bucket
    Default: RAIN_NO_CONTENT

  EmptyOnDelete:
    Type: Boolean
    Description: If true, the contents of all buckets will be permanently deleted when the stack is deleted.
    Default: false

Resources:
  LogBucket:
    Type: AWS::S3::Bucket
    Metadata:
      Comment: This bucket records access logs for the main bucket
      checkov:
        skip:
          - comment: This is the log bucket
            id: CKV_AWS_18
      guard:
        SuppressedRules:
          - S3_BUCKET_LOGGING_ENABLED
          - S3_BUCKET_REPLICATION_ENABLED
      Rain:
        Content: RAIN_NO_CONTENT
        EmptyOnDelete: !Ref EmptyOnDelete
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}
      ObjectLockConfiguration:
        ObjectLockEnabled: Enabled
        Rule:
          DefaultRetention:
            Mode: COMPLIANCE
            Years: 1
      ObjectLockEnabled: true
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      VersioningConfiguration:
        Status: Enabled

  LogBucketAccess:
    Type: !Rain::Module "bucket-policy.yaml"
    Properties:
      PolicyBucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}

  Bucket:
    Type: AWS::S3::Bucket
    Metadata:
      guard:
        SuppressedRules:
          - S3_BUCKET_DEFAULT_LOCK_ENABLED
      Rain:
        Content: !Ref Content
        EmptyOnDelete: !Ref EmptyOnDelete
        DistributionLogicalId: NONE
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}
      LoggingConfiguration:
        DestinationBucketName: !Ref LogBucket
      ObjectLockEnabled: false
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      ReplicationConfiguration:
        Role: !GetAtt ReplicationRole.Arn
        Rules:
          - Destination:
              Bucket: !GetAtt ReplicaBucket.Arn
            Status: Enabled
      VersioningConfiguration:
        Status: Enabled

  BucketAccess:
    Type: !Rain::Module "bucket-policy.yaml"
    Properties:
      PolicyBucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}

  ReplicaBucket:
    Type: AWS::S3::Bucket
    Metadata:
      Comment: This bucket is used as a target for replicas from the main bucket
      checkov:
        skip:
          - comment: This is the replica bucket
            id: CKV_AWS_18
      guard:
        SuppressedRules:
          - S3_BUCKET_DEFAULT_LOCK_ENABLED
          - S3_BUCKET_REPLICATION_ENABLED
          - S3_BUCKET_LOGGING_ENABLED
      Rain:
        Content: RAIN_NO_CONTENT
        EmptyOnDelete: !Ref EmptyOnDelete
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}
      ObjectLockEnabled: false
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      VersioningConfiguration:
        Status: Enabled

  ReplicaBucketAccess:
    Type: !Rain::Module "bucket-policy.yaml"
    Properties:
      PolicyBucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}

  ReplicationPolicy:
    Type: AWS::IAM::RolePolicy
    Properties:
      PolicyDocument:
        Statement:
          - Action:
              - s3:GetReplicationConfiguration
              - s3:ListBucket
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}
          - Action:
              - s3:GetObjectVersionForReplication
              - s3:GetObjectVersionAcl
              - s3:GetObjectVersionTagging
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}/*
          - Action:
              - s3:ReplicateObject
              - s3:ReplicateDelete
              - s3:ReplicationTags
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-replicas-${AWS::Region}-${AWS::AccountId}/*
        Version: "2012-10-17"
      PolicyName: bucket-replication-policy
      RoleName: !Ref ReplicationRole

  ReplicationRole:
    Type: AWS::IAM::Role
    Properties:
      AssumeRolePolicyDocument:
        Statement:
          - Action:
              - sts:AssumeRole
            Effect: Allow
            Principal:
              Service:
                - s3.amazonaws.com
        Version: "2012-10-17"
      Path: /

`````

### Expected (prettier)

`````yml
Description: |
  This module creates an S3 bucket that will pass common compliance checks 
  by default. It also creates an associated log bucket and replica bucket.

Parameters:
  AppName:
    Type: String
    Description: |
      This string will serve as a prefix for all resource names, which have 
      the general form of AppName-ResourceName-Region-Account.

  Content:
    Type: String
    Description: A local path to a directory that will be uploaded to the bucket
    Default: RAIN_NO_CONTENT

  EmptyOnDelete:
    Type: Boolean
    Description: If true, the contents of all buckets will be permanently deleted when the stack is deleted.
    Default: false

Resources:
  LogBucket:
    Type: AWS::S3::Bucket
    Metadata:
      Comment: This bucket records access logs for the main bucket
      checkov:
        skip:
          - comment: This is the log bucket
            id: CKV_AWS_18
      guard:
        SuppressedRules:
          - S3_BUCKET_LOGGING_ENABLED
          - S3_BUCKET_REPLICATION_ENABLED
      Rain:
        Content: RAIN_NO_CONTENT
        EmptyOnDelete: !Ref EmptyOnDelete
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}
      ObjectLockConfiguration:
        ObjectLockEnabled: Enabled
        Rule:
          DefaultRetention:
            Mode: COMPLIANCE
            Years: 1
      ObjectLockEnabled: true
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      VersioningConfiguration:
        Status: Enabled

  LogBucketAccess:
    Type: !Rain::Module "bucket-policy.yaml"
    Properties:
      PolicyBucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}

  Bucket:
    Type: AWS::S3::Bucket
    Metadata:
      guard:
        SuppressedRules:
          - S3_BUCKET_DEFAULT_LOCK_ENABLED
      Rain:
        Content: !Ref Content
        EmptyOnDelete: !Ref EmptyOnDelete
        DistributionLogicalId: NONE
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}
      LoggingConfiguration:
        DestinationBucketName: !Ref LogBucket
      ObjectLockEnabled: false
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      ReplicationConfiguration:
        Role: !GetAtt ReplicationRole.Arn
        Rules:
          - Destination:
              Bucket: !GetAtt ReplicaBucket.Arn
            Status: Enabled
      VersioningConfiguration:
        Status: Enabled

  BucketAccess:
    Type: !Rain::Module "bucket-policy.yaml"
    Properties:
      PolicyBucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}

  ReplicaBucket:
    Type: AWS::S3::Bucket
    Metadata:
      Comment: This bucket is used as a target for replicas from the main bucket
      checkov:
        skip:
          - comment: This is the replica bucket
            id: CKV_AWS_18
      guard:
        SuppressedRules:
          - S3_BUCKET_DEFAULT_LOCK_ENABLED
          - S3_BUCKET_REPLICATION_ENABLED
          - S3_BUCKET_LOGGING_ENABLED
      Rain:
        Content: RAIN_NO_CONTENT
        EmptyOnDelete: !Ref EmptyOnDelete
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}
      ObjectLockEnabled: false
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      VersioningConfiguration:
        Status: Enabled

  ReplicaBucketAccess:
    Type: !Rain::Module "bucket-policy.yaml"
    Properties:
      PolicyBucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}

  ReplicationPolicy:
    Type: AWS::IAM::RolePolicy
    Properties:
      PolicyDocument:
        Statement:
          - Action:
              - s3:GetReplicationConfiguration
              - s3:ListBucket
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}
          - Action:
              - s3:GetObjectVersionForReplication
              - s3:GetObjectVersionAcl
              - s3:GetObjectVersionTagging
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}/*
          - Action:
              - s3:ReplicateObject
              - s3:ReplicateDelete
              - s3:ReplicationTags
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-replicas-${AWS::Region}-${AWS::AccountId}/*
        Version: "2012-10-17"
      PolicyName: bucket-replication-policy
      RoleName: !Ref ReplicationRole

  ReplicationRole:
    Type: AWS::IAM::Role
    Properties:
      AssumeRolePolicyDocument:
        Statement:
          - Action:
              - sts:AssumeRole
            Effect: Allow
            Principal:
              Service:
                - s3.amazonaws.com
        Version: "2012-10-17"
      Path: /

`````

## Option 2

`````json
{"printWidth":100,"tabWidth":4,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -6,9 +6,9 @@
     AppName:
         Type: String
         Description: |
             This string will serve as a prefix for all resource names, which have 
-            the general form of AppName-ResourceName-Region-Account.
+            the general form of AppName-ResourceName-Region-Account. 
 
     Content:
         Type: String
         Description: A local path to a directory that will be uploaded to the bucket

`````

### Actual (oxfmt)

`````yml
Description: |
    This module creates an S3 bucket that will pass common compliance checks 
    by default. It also creates an associated log bucket and replica bucket.

Parameters:
    AppName:
        Type: String
        Description: |
            This string will serve as a prefix for all resource names, which have 
            the general form of AppName-ResourceName-Region-Account. 

    Content:
        Type: String
        Description: A local path to a directory that will be uploaded to the bucket
        Default: RAIN_NO_CONTENT

    EmptyOnDelete:
        Type: Boolean
        Description:
            If true, the contents of all buckets will be permanently deleted when the stack is
            deleted.
        Default: false

Resources:
    LogBucket:
        Type: AWS::S3::Bucket
        Metadata:
            Comment: This bucket records access logs for the main bucket
            checkov:
                skip:
                    - comment: This is the log bucket
                      id: CKV_AWS_18
            guard:
                SuppressedRules:
                    - S3_BUCKET_LOGGING_ENABLED
                    - S3_BUCKET_REPLICATION_ENABLED
            Rain:
                Content: RAIN_NO_CONTENT
                EmptyOnDelete: !Ref EmptyOnDelete
        Properties:
            BucketEncryption:
                ServerSideEncryptionConfiguration:
                    - ServerSideEncryptionByDefault:
                          SSEAlgorithm: AES256
            BucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}
            ObjectLockConfiguration:
                ObjectLockEnabled: Enabled
                Rule:
                    DefaultRetention:
                        Mode: COMPLIANCE
                        Years: 1
            ObjectLockEnabled: true
            PublicAccessBlockConfiguration:
                BlockPublicAcls: true
                BlockPublicPolicy: true
                IgnorePublicAcls: true
                RestrictPublicBuckets: true
            VersioningConfiguration:
                Status: Enabled

    LogBucketAccess:
        Type: !Rain::Module "bucket-policy.yaml"
        Properties:
            PolicyBucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}

    Bucket:
        Type: AWS::S3::Bucket
        Metadata:
            guard:
                SuppressedRules:
                    - S3_BUCKET_DEFAULT_LOCK_ENABLED
            Rain:
                Content: !Ref Content
                EmptyOnDelete: !Ref EmptyOnDelete
                DistributionLogicalId: NONE
        Properties:
            BucketEncryption:
                ServerSideEncryptionConfiguration:
                    - ServerSideEncryptionByDefault:
                          SSEAlgorithm: AES256
            BucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}
            LoggingConfiguration:
                DestinationBucketName: !Ref LogBucket
            ObjectLockEnabled: false
            PublicAccessBlockConfiguration:
                BlockPublicAcls: true
                BlockPublicPolicy: true
                IgnorePublicAcls: true
                RestrictPublicBuckets: true
            ReplicationConfiguration:
                Role: !GetAtt ReplicationRole.Arn
                Rules:
                    - Destination:
                          Bucket: !GetAtt ReplicaBucket.Arn
                      Status: Enabled
            VersioningConfiguration:
                Status: Enabled

    BucketAccess:
        Type: !Rain::Module "bucket-policy.yaml"
        Properties:
            PolicyBucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}

    ReplicaBucket:
        Type: AWS::S3::Bucket
        Metadata:
            Comment: This bucket is used as a target for replicas from the main bucket
            checkov:
                skip:
                    - comment: This is the replica bucket
                      id: CKV_AWS_18
            guard:
                SuppressedRules:
                    - S3_BUCKET_DEFAULT_LOCK_ENABLED
                    - S3_BUCKET_REPLICATION_ENABLED
                    - S3_BUCKET_LOGGING_ENABLED
            Rain:
                Content: RAIN_NO_CONTENT
                EmptyOnDelete: !Ref EmptyOnDelete
        Properties:
            BucketEncryption:
                ServerSideEncryptionConfiguration:
                    - ServerSideEncryptionByDefault:
                          SSEAlgorithm: AES256
            BucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}
            ObjectLockEnabled: false
            PublicAccessBlockConfiguration:
                BlockPublicAcls: true
                BlockPublicPolicy: true
                IgnorePublicAcls: true
                RestrictPublicBuckets: true
            VersioningConfiguration:
                Status: Enabled

    ReplicaBucketAccess:
        Type: !Rain::Module "bucket-policy.yaml"
        Properties:
            PolicyBucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}

    ReplicationPolicy:
        Type: AWS::IAM::RolePolicy
        Properties:
            PolicyDocument:
                Statement:
                    - Action:
                          - s3:GetReplicationConfiguration
                          - s3:ListBucket
                      Effect: Allow
                      Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}
                    - Action:
                          - s3:GetObjectVersionForReplication
                          - s3:GetObjectVersionAcl
                          - s3:GetObjectVersionTagging
                      Effect: Allow
                      Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}/*
                    - Action:
                          - s3:ReplicateObject
                          - s3:ReplicateDelete
                          - s3:ReplicationTags
                      Effect: Allow
                      Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-replicas-${AWS::Region}-${AWS::AccountId}/*
                Version: "2012-10-17"
            PolicyName: bucket-replication-policy
            RoleName: !Ref ReplicationRole

    ReplicationRole:
        Type: AWS::IAM::Role
        Properties:
            AssumeRolePolicyDocument:
                Statement:
                    - Action:
                          - sts:AssumeRole
                      Effect: Allow
                      Principal:
                          Service:
                              - s3.amazonaws.com
                Version: "2012-10-17"
            Path: /

`````

### Expected (prettier)

`````yml
Description: |
    This module creates an S3 bucket that will pass common compliance checks 
    by default. It also creates an associated log bucket and replica bucket.

Parameters:
    AppName:
        Type: String
        Description: |
            This string will serve as a prefix for all resource names, which have 
            the general form of AppName-ResourceName-Region-Account.

    Content:
        Type: String
        Description: A local path to a directory that will be uploaded to the bucket
        Default: RAIN_NO_CONTENT

    EmptyOnDelete:
        Type: Boolean
        Description:
            If true, the contents of all buckets will be permanently deleted when the stack is
            deleted.
        Default: false

Resources:
    LogBucket:
        Type: AWS::S3::Bucket
        Metadata:
            Comment: This bucket records access logs for the main bucket
            checkov:
                skip:
                    - comment: This is the log bucket
                      id: CKV_AWS_18
            guard:
                SuppressedRules:
                    - S3_BUCKET_LOGGING_ENABLED
                    - S3_BUCKET_REPLICATION_ENABLED
            Rain:
                Content: RAIN_NO_CONTENT
                EmptyOnDelete: !Ref EmptyOnDelete
        Properties:
            BucketEncryption:
                ServerSideEncryptionConfiguration:
                    - ServerSideEncryptionByDefault:
                          SSEAlgorithm: AES256
            BucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}
            ObjectLockConfiguration:
                ObjectLockEnabled: Enabled
                Rule:
                    DefaultRetention:
                        Mode: COMPLIANCE
                        Years: 1
            ObjectLockEnabled: true
            PublicAccessBlockConfiguration:
                BlockPublicAcls: true
                BlockPublicPolicy: true
                IgnorePublicAcls: true
                RestrictPublicBuckets: true
            VersioningConfiguration:
                Status: Enabled

    LogBucketAccess:
        Type: !Rain::Module "bucket-policy.yaml"
        Properties:
            PolicyBucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}

    Bucket:
        Type: AWS::S3::Bucket
        Metadata:
            guard:
                SuppressedRules:
                    - S3_BUCKET_DEFAULT_LOCK_ENABLED
            Rain:
                Content: !Ref Content
                EmptyOnDelete: !Ref EmptyOnDelete
                DistributionLogicalId: NONE
        Properties:
            BucketEncryption:
                ServerSideEncryptionConfiguration:
                    - ServerSideEncryptionByDefault:
                          SSEAlgorithm: AES256
            BucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}
            LoggingConfiguration:
                DestinationBucketName: !Ref LogBucket
            ObjectLockEnabled: false
            PublicAccessBlockConfiguration:
                BlockPublicAcls: true
                BlockPublicPolicy: true
                IgnorePublicAcls: true
                RestrictPublicBuckets: true
            ReplicationConfiguration:
                Role: !GetAtt ReplicationRole.Arn
                Rules:
                    - Destination:
                          Bucket: !GetAtt ReplicaBucket.Arn
                      Status: Enabled
            VersioningConfiguration:
                Status: Enabled

    BucketAccess:
        Type: !Rain::Module "bucket-policy.yaml"
        Properties:
            PolicyBucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}

    ReplicaBucket:
        Type: AWS::S3::Bucket
        Metadata:
            Comment: This bucket is used as a target for replicas from the main bucket
            checkov:
                skip:
                    - comment: This is the replica bucket
                      id: CKV_AWS_18
            guard:
                SuppressedRules:
                    - S3_BUCKET_DEFAULT_LOCK_ENABLED
                    - S3_BUCKET_REPLICATION_ENABLED
                    - S3_BUCKET_LOGGING_ENABLED
            Rain:
                Content: RAIN_NO_CONTENT
                EmptyOnDelete: !Ref EmptyOnDelete
        Properties:
            BucketEncryption:
                ServerSideEncryptionConfiguration:
                    - ServerSideEncryptionByDefault:
                          SSEAlgorithm: AES256
            BucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}
            ObjectLockEnabled: false
            PublicAccessBlockConfiguration:
                BlockPublicAcls: true
                BlockPublicPolicy: true
                IgnorePublicAcls: true
                RestrictPublicBuckets: true
            VersioningConfiguration:
                Status: Enabled

    ReplicaBucketAccess:
        Type: !Rain::Module "bucket-policy.yaml"
        Properties:
            PolicyBucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}

    ReplicationPolicy:
        Type: AWS::IAM::RolePolicy
        Properties:
            PolicyDocument:
                Statement:
                    - Action:
                          - s3:GetReplicationConfiguration
                          - s3:ListBucket
                      Effect: Allow
                      Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}
                    - Action:
                          - s3:GetObjectVersionForReplication
                          - s3:GetObjectVersionAcl
                          - s3:GetObjectVersionTagging
                      Effect: Allow
                      Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}/*
                    - Action:
                          - s3:ReplicateObject
                          - s3:ReplicateDelete
                          - s3:ReplicationTags
                      Effect: Allow
                      Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-replicas-${AWS::Region}-${AWS::AccountId}/*
                Version: "2012-10-17"
            PolicyName: bucket-replication-policy
            RoleName: !Ref ReplicationRole

    ReplicationRole:
        Type: AWS::IAM::Role
        Properties:
            AssumeRolePolicyDocument:
                Statement:
                    - Action:
                          - sts:AssumeRole
                      Effect: Allow
                      Principal:
                          Service:
                              - s3.amazonaws.com
                Version: "2012-10-17"
            Path: /

`````

## Option 3

`````json
{"printWidth":120,"singleQuote":true,"bracketSpacing":false,"trailingComma":"none"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -6,9 +6,9 @@
   AppName:
     Type: String
     Description: |
       This string will serve as a prefix for all resource names, which have 
-      the general form of AppName-ResourceName-Region-Account.
+      the general form of AppName-ResourceName-Region-Account. 
 
   Content:
     Type: String
     Description: A local path to a directory that will be uploaded to the bucket

`````

### Actual (oxfmt)

`````yml
Description: |
  This module creates an S3 bucket that will pass common compliance checks 
  by default. It also creates an associated log bucket and replica bucket.

Parameters:
  AppName:
    Type: String
    Description: |
      This string will serve as a prefix for all resource names, which have 
      the general form of AppName-ResourceName-Region-Account. 

  Content:
    Type: String
    Description: A local path to a directory that will be uploaded to the bucket
    Default: RAIN_NO_CONTENT

  EmptyOnDelete:
    Type: Boolean
    Description: If true, the contents of all buckets will be permanently deleted when the stack is deleted.
    Default: false

Resources:
  LogBucket:
    Type: AWS::S3::Bucket
    Metadata:
      Comment: This bucket records access logs for the main bucket
      checkov:
        skip:
          - comment: This is the log bucket
            id: CKV_AWS_18
      guard:
        SuppressedRules:
          - S3_BUCKET_LOGGING_ENABLED
          - S3_BUCKET_REPLICATION_ENABLED
      Rain:
        Content: RAIN_NO_CONTENT
        EmptyOnDelete: !Ref EmptyOnDelete
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}
      ObjectLockConfiguration:
        ObjectLockEnabled: Enabled
        Rule:
          DefaultRetention:
            Mode: COMPLIANCE
            Years: 1
      ObjectLockEnabled: true
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      VersioningConfiguration:
        Status: Enabled

  LogBucketAccess:
    Type: !Rain::Module 'bucket-policy.yaml'
    Properties:
      PolicyBucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}

  Bucket:
    Type: AWS::S3::Bucket
    Metadata:
      guard:
        SuppressedRules:
          - S3_BUCKET_DEFAULT_LOCK_ENABLED
      Rain:
        Content: !Ref Content
        EmptyOnDelete: !Ref EmptyOnDelete
        DistributionLogicalId: NONE
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}
      LoggingConfiguration:
        DestinationBucketName: !Ref LogBucket
      ObjectLockEnabled: false
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      ReplicationConfiguration:
        Role: !GetAtt ReplicationRole.Arn
        Rules:
          - Destination:
              Bucket: !GetAtt ReplicaBucket.Arn
            Status: Enabled
      VersioningConfiguration:
        Status: Enabled

  BucketAccess:
    Type: !Rain::Module 'bucket-policy.yaml'
    Properties:
      PolicyBucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}

  ReplicaBucket:
    Type: AWS::S3::Bucket
    Metadata:
      Comment: This bucket is used as a target for replicas from the main bucket
      checkov:
        skip:
          - comment: This is the replica bucket
            id: CKV_AWS_18
      guard:
        SuppressedRules:
          - S3_BUCKET_DEFAULT_LOCK_ENABLED
          - S3_BUCKET_REPLICATION_ENABLED
          - S3_BUCKET_LOGGING_ENABLED
      Rain:
        Content: RAIN_NO_CONTENT
        EmptyOnDelete: !Ref EmptyOnDelete
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}
      ObjectLockEnabled: false
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      VersioningConfiguration:
        Status: Enabled

  ReplicaBucketAccess:
    Type: !Rain::Module 'bucket-policy.yaml'
    Properties:
      PolicyBucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}

  ReplicationPolicy:
    Type: AWS::IAM::RolePolicy
    Properties:
      PolicyDocument:
        Statement:
          - Action:
              - s3:GetReplicationConfiguration
              - s3:ListBucket
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}
          - Action:
              - s3:GetObjectVersionForReplication
              - s3:GetObjectVersionAcl
              - s3:GetObjectVersionTagging
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}/*
          - Action:
              - s3:ReplicateObject
              - s3:ReplicateDelete
              - s3:ReplicationTags
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-replicas-${AWS::Region}-${AWS::AccountId}/*
        Version: '2012-10-17'
      PolicyName: bucket-replication-policy
      RoleName: !Ref ReplicationRole

  ReplicationRole:
    Type: AWS::IAM::Role
    Properties:
      AssumeRolePolicyDocument:
        Statement:
          - Action:
              - sts:AssumeRole
            Effect: Allow
            Principal:
              Service:
                - s3.amazonaws.com
        Version: '2012-10-17'
      Path: /

`````

### Expected (prettier)

`````yml
Description: |
  This module creates an S3 bucket that will pass common compliance checks 
  by default. It also creates an associated log bucket and replica bucket.

Parameters:
  AppName:
    Type: String
    Description: |
      This string will serve as a prefix for all resource names, which have 
      the general form of AppName-ResourceName-Region-Account.

  Content:
    Type: String
    Description: A local path to a directory that will be uploaded to the bucket
    Default: RAIN_NO_CONTENT

  EmptyOnDelete:
    Type: Boolean
    Description: If true, the contents of all buckets will be permanently deleted when the stack is deleted.
    Default: false

Resources:
  LogBucket:
    Type: AWS::S3::Bucket
    Metadata:
      Comment: This bucket records access logs for the main bucket
      checkov:
        skip:
          - comment: This is the log bucket
            id: CKV_AWS_18
      guard:
        SuppressedRules:
          - S3_BUCKET_LOGGING_ENABLED
          - S3_BUCKET_REPLICATION_ENABLED
      Rain:
        Content: RAIN_NO_CONTENT
        EmptyOnDelete: !Ref EmptyOnDelete
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}
      ObjectLockConfiguration:
        ObjectLockEnabled: Enabled
        Rule:
          DefaultRetention:
            Mode: COMPLIANCE
            Years: 1
      ObjectLockEnabled: true
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      VersioningConfiguration:
        Status: Enabled

  LogBucketAccess:
    Type: !Rain::Module 'bucket-policy.yaml'
    Properties:
      PolicyBucketName: !Sub ${AppName}-logs-${AWS::Region}-${AWS::AccountId}

  Bucket:
    Type: AWS::S3::Bucket
    Metadata:
      guard:
        SuppressedRules:
          - S3_BUCKET_DEFAULT_LOCK_ENABLED
      Rain:
        Content: !Ref Content
        EmptyOnDelete: !Ref EmptyOnDelete
        DistributionLogicalId: NONE
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}
      LoggingConfiguration:
        DestinationBucketName: !Ref LogBucket
      ObjectLockEnabled: false
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      ReplicationConfiguration:
        Role: !GetAtt ReplicationRole.Arn
        Rules:
          - Destination:
              Bucket: !GetAtt ReplicaBucket.Arn
            Status: Enabled
      VersioningConfiguration:
        Status: Enabled

  BucketAccess:
    Type: !Rain::Module 'bucket-policy.yaml'
    Properties:
      PolicyBucketName: !Sub ${AppName}-${AWS::Region}-${AWS::AccountId}

  ReplicaBucket:
    Type: AWS::S3::Bucket
    Metadata:
      Comment: This bucket is used as a target for replicas from the main bucket
      checkov:
        skip:
          - comment: This is the replica bucket
            id: CKV_AWS_18
      guard:
        SuppressedRules:
          - S3_BUCKET_DEFAULT_LOCK_ENABLED
          - S3_BUCKET_REPLICATION_ENABLED
          - S3_BUCKET_LOGGING_ENABLED
      Rain:
        Content: RAIN_NO_CONTENT
        EmptyOnDelete: !Ref EmptyOnDelete
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      BucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}
      ObjectLockEnabled: false
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      VersioningConfiguration:
        Status: Enabled

  ReplicaBucketAccess:
    Type: !Rain::Module 'bucket-policy.yaml'
    Properties:
      PolicyBucketName: !Sub ${AppName}-replicas-${AWS::Region}-${AWS::AccountId}

  ReplicationPolicy:
    Type: AWS::IAM::RolePolicy
    Properties:
      PolicyDocument:
        Statement:
          - Action:
              - s3:GetReplicationConfiguration
              - s3:ListBucket
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}
          - Action:
              - s3:GetObjectVersionForReplication
              - s3:GetObjectVersionAcl
              - s3:GetObjectVersionTagging
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-${AWS::Region}-${AWS::AccountId}/*
          - Action:
              - s3:ReplicateObject
              - s3:ReplicateDelete
              - s3:ReplicationTags
            Effect: Allow
            Resource: !Sub arn:${AWS::Partition}:s3:::${AppName}-replicas-${AWS::Region}-${AWS::AccountId}/*
        Version: '2012-10-17'
      PolicyName: bucket-replication-policy
      RoleName: !Ref ReplicationRole

  ReplicationRole:
    Type: AWS::IAM::Role
    Properties:
      AssumeRolePolicyDocument:
        Statement:
          - Action:
              - sts:AssumeRole
            Effect: Allow
            Principal:
              Service:
                - s3.amazonaws.com
        Version: '2012-10-17'
      Path: /

`````
