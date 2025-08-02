# \TasksApi

All URIs are relative to *http://localhost:1234*

Method | HTTP request | Description
------------- | ------------- | -------------
[**tasks_get**](TasksApi.md#tasks_get) | **GET** /tasks | 
[**tasks_post**](TasksApi.md#tasks_post) | **POST** /tasks | 
[**tasks_task_id_delete**](TasksApi.md#tasks_task_id_delete) | **DELETE** /tasks/{taskId} | 
[**tasks_task_id_put**](TasksApi.md#tasks_task_id_put) | **PUT** /tasks/{taskId} | 



## tasks_get

> Vec<models::Task> tasks_get()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Task>**](Task.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tasks_post

> tasks_post(tasks_post_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tasks_post_request** | [**TasksPostRequest**](TasksPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tasks_task_id_delete

> tasks_task_id_delete(task_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**task_id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tasks_task_id_put

> tasks_task_id_put(task_id, tasks_task_id_put_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**task_id** | **uuid::Uuid** |  | [required] |
**tasks_task_id_put_request** | [**TasksTaskIdPutRequest**](TasksTaskIdPutRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

