# \SecretsApi

All URIs are relative to *http://localhost:1234*

Method | HTTP request | Description
------------- | ------------- | -------------
[**secrets_get**](SecretsApi.md#secrets_get) | **GET** /secrets | 
[**secrets_post**](SecretsApi.md#secrets_post) | **POST** /secrets | 



## secrets_get

> Vec<models::Secret> secrets_get()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Secret>**](Secret.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## secrets_post

> secrets_post(secrets_post_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**secrets_post_request** | [**SecretsPostRequest**](SecretsPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

