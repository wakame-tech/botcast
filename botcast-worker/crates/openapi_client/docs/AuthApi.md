# \AuthApi

All URIs are relative to *http://localhost:1234*

Method | HTTP request | Description
------------- | ------------- | -------------
[**me_get**](AuthApi.md#me_get) | **GET** /me | 
[**sign_in_post**](AuthApi.md#sign_in_post) | **POST** /signIn | Sign in



## me_get

> models::User me_get()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::User**](User.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sign_in_post

> models::SignInPost200Response sign_in_post(sign_in_post_request)
Sign in

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sign_in_post_request** | [**SignInPostRequest**](SignInPostRequest.md) |  | [required] |

### Return type

[**models::SignInPost200Response**](_signIn_post_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

