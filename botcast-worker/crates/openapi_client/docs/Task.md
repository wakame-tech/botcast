# Task

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**status** | [**models::TaskStatus**](TaskStatus.md) |  | 
**args** | [**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md) |  | 
**user_id** | Option<[**uuid::Uuid**](uuid::Uuid.md)> |  | [optional]
**execute_after** | **String** |  | 
**executed_at** | Option<**String**> |  | [optional]
**executed_finished_at** | Option<**String**> |  | [optional]
**result** | Option<[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)> |  | [optional]
**cron** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


