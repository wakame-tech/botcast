# \PodcastsApi

All URIs are relative to *http://localhost:1234*

Method | HTTP request | Description
------------- | ------------- | -------------
[**podcast_podcast_id_delete**](PodcastsApi.md#podcast_podcast_id_delete) | **DELETE** /podcast/{podcastId} | 
[**podcast_podcast_id_get**](PodcastsApi.md#podcast_podcast_id_get) | **GET** /podcast/{podcastId} | 
[**podcast_podcast_id_put**](PodcastsApi.md#podcast_podcast_id_put) | **PUT** /podcast/{podcastId} | 
[**podcasts_get**](PodcastsApi.md#podcasts_get) | **GET** /podcasts | 
[**podcasts_post**](PodcastsApi.md#podcasts_post) | **POST** /podcasts | 
[**top_podcasts_get**](PodcastsApi.md#top_podcasts_get) | **GET** /topPodcasts | 



## podcast_podcast_id_delete

> podcast_podcast_id_delete(podcast_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**podcast_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## podcast_podcast_id_get

> models::PodcastPodcastIdGet200Response podcast_podcast_id_get(podcast_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**podcast_id** | **String** |  | [required] |

### Return type

[**models::PodcastPodcastIdGet200Response**](_podcast__podcastId__get_200_response.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## podcast_podcast_id_put

> podcast_podcast_id_put(podcast_id, podcasts_post_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**podcast_id** | **String** |  | [required] |
**podcasts_post_request** | [**PodcastsPostRequest**](PodcastsPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## podcasts_get

> Vec<models::Podcast> podcasts_get()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Podcast>**](Podcast.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## podcasts_post

> podcasts_post(podcasts_post_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**podcasts_post_request** | [**PodcastsPostRequest**](PodcastsPostRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## top_podcasts_get

> Vec<models::Podcast> top_podcasts_get()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Podcast>**](Podcast.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

