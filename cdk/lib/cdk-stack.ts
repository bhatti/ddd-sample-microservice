import {Stack, RemovalPolicy, StackProps, Duration, Aws} from 'aws-cdk-lib';
import {
    IResource,
    LambdaIntegration,
    MockIntegration,
    PassthroughBehavior,
    Resource,
    RestApi
} from 'aws-cdk-lib/aws-apigateway';
import {AttributeType, Table, ProjectionType} from 'aws-cdk-lib/aws-dynamodb';
import {Construct} from 'constructs';
import * as path from 'path';
import {Code, Function, Runtime, FunctionUrlAuthType} from 'aws-cdk-lib/aws-lambda';
import {CfnOutput} from "aws-cdk-lib";
import {Alarm, Dashboard, GraphWidget, LogQueryWidget, TextWidget, TreatMissingData} from "aws-cdk-lib/aws-cloudwatch";
import {AccountPrincipal, PolicyDocument, PolicyStatement, ServicePrincipal} from "aws-cdk-lib/aws-iam";

export class CdkStack extends Stack {
    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props);

        const stack = this;
        const booksDynamoTable = CdkStack.buildBooksTable(stack);
        const partiesDynamoTable = CdkStack.buildPartiesTable(stack);
        const checkoutDynamoTable = CdkStack.buildCheckoutTable(stack);
        const holdDynamoTable = CdkStack.buildHoldTable(stack);

        const apiPolicy = new PolicyDocument({
            statements: [
                new PolicyStatement({
                    actions: ["execute-api:Invoke"],
                    principals: [
                        new AccountPrincipal(this.account),
                        // for cross-account access add other account principals here
                    ],
                }),
            ],
        });

        // Create an API Gateway resource for each of the CRUD operations
        const api = new RestApi(stack, 'lms', {
            restApiName: 'Library Management System API',
            policy: apiPolicy, // our API policy that allows cross-account access
        });

        const tables = [booksDynamoTable, partiesDynamoTable, checkoutDynamoTable, holdDynamoTable];
        const methods = ['POST', 'PUT', 'DELETE', 'GET'];

        // Create CloudWatch Dashboard
        const dashboardName = 'LibraryManageSystemDashboard';
        const dashboard = new Dashboard(this, dashboardName, {
            dashboardName: dashboardName
        })

        const principal = new ServicePrincipal('lms-service');
        // This can be broken out for each DDB-table/API/Lambda but we are making assumption that we will
        // create CREDL based REST APIs.
        const lambdaBins = {'books': 'catalog', 'parties': 'patrons', 'checkout': 'checkout', 'hold': 'hold'};
        tables.forEach((table) => {
            const tableName = table.node.id;
            // @ts-ignore
            const lambdaBin = lambdaBins[tableName];
            const resource = api.root.addResource(tableName);
            methods.forEach((method) => {
                const handlerName = method + '-' + tableName;
                const handler = CdkStack.addLambda(stack, method, tableName, table, resource, handlerName, lambdaBin);
                handler.grantInvoke(principal);
                handler.addPermission('lms-service:invoke', {
                    principal: principal,
                });
                this.addCloudWatch(dashboard, handler, handlerName);
                if (method === 'PUT' || method === 'DELETE' || method === 'GET') {
                    const childResource = resource.addResource('{id}');
                    childResource.addMethod(method);
                }
            });
            // Add CORS options
            addCorsOptions(resource);
        });

        // Generate Outputs
        const cloudwatchDashboardURL = `https://${Aws.REGION}.console.aws.amazon.com/cloudwatch/home?region=${Aws.REGION}#dashboards:name=${dashboardName}`;
        new CfnOutput(this, `DashboardOutput`, {
            value: cloudwatchDashboardURL,
            description: 'URL of Sample CloudWatch Dashboard',
            exportName: 'SampleCloudWatchDashboardURL'
        });
    }

    private static addLambda(stack: Stack, method: string, tableName: string, table: Table,
                             resource: Resource, handlerName: string, lambdaBin: string) {
        const handler = new Function(stack, handlerName, {
            // The source code of your Lambda function. You can point to a file in an Amazon Simple Storage Service (Amazon S3) bucket
            // or specify your source code as inline text.
            code: Code.fromAsset(path.join(__dirname, '../../lms', 'target/lambda/' + lambdaBin)),
            // The runtime environment for the Lambda function that you are uploading.
            // For valid values, see the Runtime property in the AWS Lambda Developer Guide.
            // Use Runtime.FROM_IMAGE when defining a function from a Docker image.
            runtime: Runtime.PROVIDED_AL2,
            handler: 'does_not_matter',
            // The function execution time (in seconds) after which Lambda terminates the function.
            functionName: handlerName,
            memorySize: 512,
            timeout: Duration.seconds(30),
            environment: {
                TABLE_NAME: tableName,
                INDEX_NAME: tableName + "_status",
            }
        });

        const fnUrl = handler.addFunctionUrl({authType: FunctionUrlAuthType.AWS_IAM});
        new CfnOutput(stack, `${handlerName}Url`, {
            value: fnUrl.url, // The .url attributes will return the unique Function URL
        });

        table.grantReadWriteData(handler);
        const lambdaIntegration = new LambdaIntegration(handler);
        resource.addMethod(method, lambdaIntegration);
        return handler;
    }

    private addCloudWatch(dashboard: Dashboard, handler: Function, handlerName: string) {
        if (handler.timeout) {
            new Alarm(this, `${handlerName}Alarm`, {
                metric: handler.metricDuration().with({
                    statistic: 'Maximum',
                }),
                evaluationPeriods: 1,
                datapointsToAlarm: 1,
                threshold: handler.timeout.toMilliseconds(),
                treatMissingData: TreatMissingData.IGNORE,
                alarmName: `${handlerName} Timeout`,
            });
        }
        // Create Title for Dashboard
        dashboard.addWidgets(new TextWidget({
            markdown: `# Dashboard: ${handler.functionName}`,
            height: 1,
            width: 24
        }))

        // Create CloudWatch Dashboard Widgets: Errors, Invocations, Duration, Throttles
        dashboard.addWidgets(new GraphWidget({
            title: "Invocations",
            left: [handler.metricInvocations()],
            width: 24
        }))

        dashboard.addWidgets(new GraphWidget({
            title: "Errors",
            left: [handler.metricErrors()],
            width: 24
        }))

        dashboard.addWidgets(new GraphWidget({
            title: "Duration",
            left: [handler.metricDuration()],
            width: 24
        }))

        dashboard.addWidgets(new GraphWidget({
            title: "Throttles",
            left: [handler.metricThrottles()],
            width: 24
        }))

        // Create Widget to show last 20 Log Entries
        dashboard.addWidgets(new LogQueryWidget({
            logGroupNames: [handler.logGroup.logGroupName],
            queryLines: [
                "fields @timestamp, @message",
                "sort @timestamp desc",
                "limit 20"],
            width: 24,
        }))

        new CfnOutput(this, `LambdaName${handlerName}`, {
            value: handler.functionName,
            description: `Name of the ${handlerName} Lambda Function`,
            exportName: `LambdaName${handlerName}`
        });
    }

    private static buildHoldTable(stack: Stack) {
        const holdDynamoTable = new Table(stack, 'hold', {
            partitionKey: {
                name: 'hold_id',
                type: AttributeType.STRING
            },
            readCapacity: 10,
            writeCapacity: 10,
            pointInTimeRecovery: true,
            tableName: 'hold',
            removalPolicy: RemovalPolicy.DESTROY, // NOT recommended for production code
        });
        holdDynamoTable.addGlobalSecondaryIndex({
            indexName: 'hold_ndx',
            partitionKey: {name: 'hold_status', type: AttributeType.STRING},
            sortKey: {name: 'patron_id', type: AttributeType.STRING},
            readCapacity: 10,
            writeCapacity: 10,
            projectionType: ProjectionType.ALL,
        });
        return holdDynamoTable;
    }

    private static buildCheckoutTable(stack: Stack) {
        const checkoutDynamoTable = new Table(stack, 'checkout', {
            partitionKey: {
                name: 'checkout_id',
                type: AttributeType.STRING
            },
            readCapacity: 10,
            writeCapacity: 10,
            pointInTimeRecovery: true,
            tableName: 'checkout',
            removalPolicy: RemovalPolicy.DESTROY, // NOT recommended for production code
        });
        checkoutDynamoTable.addGlobalSecondaryIndex({
            indexName: 'checkout_ndx',
            partitionKey: {name: 'checkout_status', type: AttributeType.STRING},
            sortKey: {name: 'patron_id', type: AttributeType.STRING},
            readCapacity: 10,
            writeCapacity: 10,
            projectionType: ProjectionType.ALL,
        });
        return checkoutDynamoTable;
    }

    private static buildPartiesTable(stack: Stack) {
        // Party kind can be: patron, author, publisher, employee (see Party pattern https://martinfowler.com/apsupp/accountability.pdf)
        const partiesDynamoTable = new Table(stack, 'parties', {
            partitionKey: {
                name: 'party_id',
                type: AttributeType.STRING
            },
            readCapacity: 10,
            writeCapacity: 10,
            pointInTimeRecovery: true,
            tableName: 'parties',
            removalPolicy: RemovalPolicy.DESTROY, // NOT recommended for production code
        });
        partiesDynamoTable.addGlobalSecondaryIndex({
            indexName: 'parties_ndx',
            partitionKey: {name: 'kind', type: AttributeType.STRING},
            sortKey: {name: 'email', type: AttributeType.STRING},
            readCapacity: 10,
            writeCapacity: 10,
            projectionType: ProjectionType.ALL,
        });
        return partiesDynamoTable;
    }

    private static buildBooksTable(stack: Stack) {
        const booksDynamoTable = new Table(stack, 'books', {
            partitionKey: {
                name: 'book_id',
                type: AttributeType.STRING
            },
            readCapacity: 10,
            writeCapacity: 10,
            pointInTimeRecovery: true,
            tableName: 'books',

            /**
             *  The default removal policy is RETAIN, which means that cdk destroy will not attempt to delete
             * the new table, and it will remain in your account until manually deleted. By setting the policy to
             * DESTROY, cdk destroy will delete the table (even if it has data in it)
             */
            removalPolicy: RemovalPolicy.DESTROY, // NOT recommended for production code
        });

        booksDynamoTable.addGlobalSecondaryIndex({
            indexName: 'books_ndx',
            partitionKey: {name: 'book_status', type: AttributeType.STRING},
            sortKey: {name: 'isbn', type: AttributeType.STRING},
            readCapacity: 10,
            writeCapacity: 10,
            projectionType: ProjectionType.ALL,
        });
        return booksDynamoTable;
    }
}

export function addCorsOptions(apiResource: IResource) {
    apiResource.addMethod('OPTIONS', new MockIntegration({
        integrationResponses: [{
            statusCode: '200',
            responseParameters: {
                'method.response.header.Access-Control-Allow-Headers': "'Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token,X-Amz-User-Agent'",
                'method.response.header.Access-Control-Allow-Origin': "'*'",
                'method.response.header.Access-Control-Allow-Credentials': "'false'",
                'method.response.header.Access-Control-Allow-Methods': "'OPTIONS,GET,PUT,POST,DELETE'",
            },
        }],
        passthroughBehavior: PassthroughBehavior.NEVER,
        requestTemplates: {
            "application/json": "{\"statusCode\": 200}"
        },
    }), {
        methodResponses: [{
            statusCode: '200',
            responseParameters: {
                'method.response.header.Access-Control-Allow-Headers': true,
                'method.response.header.Access-Control-Allow-Methods': true,
                'method.response.header.Access-Control-Allow-Credentials': true,
                'method.response.header.Access-Control-Allow-Origin': true,
            },
        }]
    })
}
