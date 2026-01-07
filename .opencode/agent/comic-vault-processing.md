---
name: comic-vault-processing
description: Azure Durable Functions patterns. Load when working on comic processing pipelines or background jobs.
---

# Comic Vault Processing Skill

**Purpose**: Guide Azure Durable Functions development for comic processing pipelines.

## When to Load
- Creating/modifying processing jobs
- Azure Functions development
- Orchestration workflows
- Background job handling

## Architecture Overview

Comic Vault uses Azure Durable Functions for multi-step comic processing:

```
Blob Upload Trigger
     ↓
Orchestrator (coordinates all steps)
     ↓
Activities (individual processing steps)
     ├── ExtractPages
     ├── ExtractText (OCR)
     ├── FetchMetadata (Comic Vine API)
     ├── DetectHeroes (Vision AI)
     ├── GenerateSummary
     └── GenerateEmbeddings
```

## Project Structure

```
src/ComicRag.Functions/
├── Triggers/
│   ├── ComicUploadTrigger.cs    # Blob trigger
│   └── CoverOptimizationTrigger.cs
├── Orchestrators/
│   └── ComicProcessingOrchestrator.cs
├── Activities/
│   ├── ExtractPagesActivity.cs
│   ├── ExtractTextActivity.cs
│   ├── FetchMetadataActivity.cs
│   ├── DetectHeroesActivity.cs
│   ├── GenerateSummaryActivity.cs
│   └── GenerateEmbeddingsActivity.cs
├── Models/
│   ├── ComicProcessingInput.cs
│   └── ProcessingStatus.cs
└── Program.cs
```

## Core Patterns

### 1. Blob Trigger

```csharp
// Triggers/ComicUploadTrigger.cs
public class ComicUploadTrigger(ILogger<ComicUploadTrigger> logger)
{
    [Function(nameof(ComicUploadTrigger))]
    public async Task Run(
        [BlobTrigger("comic-uploads/{name}", Connection = "AzureWebJobsStorage")]
        BlobClient blob,
        string name,
        [DurableClient] DurableTaskClient client)
    {
        logger.LogInformation("Processing blob: {Name}", name);

        // Parse metadata from blob
        var properties = await blob.GetPropertiesAsync();
        var metadata = properties.Value.Metadata;

        var input = new ComicProcessingInput
        {
            BlobName = name,
            ComicId = Guid.Parse(metadata["ComicId"]),
            UserId = Guid.Parse(metadata["UserId"])
        };

        // Start orchestration
        var instanceId = await client.ScheduleNewOrchestrationInstanceAsync(
            nameof(ComicProcessingOrchestrator),
            input);

        logger.LogInformation("Started orchestration: {InstanceId}", instanceId);
    }
}
```

### 2. Orchestrator

```csharp
// Orchestrators/ComicProcessingOrchestrator.cs
public class ComicProcessingOrchestrator
{
    [Function(nameof(ComicProcessingOrchestrator))]
    public async Task<ProcessingResult> Run(
        [OrchestrationTrigger] TaskOrchestrationContext context)
    {
        var input = context.GetInput<ComicProcessingInput>()!;
        var logger = context.CreateReplaySafeLogger<ComicProcessingOrchestrator>();

        try
        {
            // Step 1: Extract pages from archive
            logger.LogInformation("Extracting pages...");
            var pages = await context.CallActivityAsync<List<PageInfo>>(
                nameof(ExtractPagesActivity),
                input);

            await UpdateStatus(context, input.ComicId, "Extracting text...");

            // Step 2: Parallel operations - OCR and metadata fetch
            var ocrTask = context.CallActivityAsync<OcrResult>(
                nameof(ExtractTextActivity),
                new ExtractTextInput(input.ComicId, pages));

            var metadataTask = context.CallActivityAsync<ComicMetadata>(
                nameof(FetchMetadataActivity),
                input);

            await Task.WhenAll(ocrTask, metadataTask);

            var ocrResult = ocrTask.Result;
            var metadata = metadataTask.Result;

            await UpdateStatus(context, input.ComicId, "Detecting heroes...");

            // Step 3: Hero detection (requires extracted pages)
            var heroes = await context.CallActivityAsync<List<HeroDetection>>(
                nameof(DetectHeroesActivity),
                new DetectHeroesInput(input.ComicId, pages));

            await UpdateStatus(context, input.ComicId, "Generating summary...");

            // Step 4: Generate summary
            var summary = await context.CallActivityAsync<string>(
                nameof(GenerateSummaryActivity),
                new SummaryInput(input.ComicId, ocrResult.Text, metadata));

            await UpdateStatus(context, input.ComicId, "Generating embeddings...");

            // Step 5: Generate embeddings for RAG
            await context.CallActivityAsync(
                nameof(GenerateEmbeddingsActivity),
                new EmbeddingsInput(input.ComicId, pages, ocrResult));

            await UpdateStatus(context, input.ComicId, "Complete");

            return new ProcessingResult(
                Success: true,
                ComicId: input.ComicId,
                PageCount: pages.Count,
                HeroCount: heroes.Count
            );
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Processing failed for comic {ComicId}", input.ComicId);
            await UpdateStatus(context, input.ComicId, "Failed", ex.Message);

            return new ProcessingResult(
                Success: false,
                ComicId: input.ComicId,
                Error: ex.Message
            );
        }
    }

    private static async Task UpdateStatus(
        TaskOrchestrationContext context,
        Guid comicId,
        string status,
        string? error = null)
    {
        await context.CallActivityAsync(
            nameof(UpdateJobStatusActivity),
            new StatusUpdate(comicId, status, error));
    }
}
```

### 3. Activity Functions

```csharp
// Activities/ExtractPagesActivity.cs
public class ExtractPagesActivity(
    IBlobStorageService blobStorage,
    IComicExtractor extractor)
{
    [Function(nameof(ExtractPagesActivity))]
    public async Task<List<PageInfo>> Run(
        [ActivityTrigger] ComicProcessingInput input)
    {
        // Download blob
        var blobData = await blobStorage.DownloadAsync(
            $"comic-uploads/{input.BlobName}");

        // Extract pages from CBZ/CBR/PDF
        var pages = await extractor.ExtractPagesAsync(blobData, input.ComicId);

        // Upload extracted pages
        var pageInfos = new List<PageInfo>();
        foreach (var (pageNumber, imageData) in pages)
        {
            var path = $"comics/{input.ComicId}/pages/{pageNumber:D4}.jpg";
            await blobStorage.UploadAsync(path, imageData);

            pageInfos.Add(new PageInfo(pageNumber, path));
        }

        return pageInfos;
    }
}

// Activities/DetectHeroesActivity.cs
public class DetectHeroesActivity(
    ILLMService llm,
    IBlobStorageService blobStorage)
{
    [Function(nameof(DetectHeroesActivity))]
    public async Task<List<HeroDetection>> Run(
        [ActivityTrigger] DetectHeroesInput input)
    {
        var allDetections = new List<HeroDetection>();

        foreach (var page in input.Pages)
        {
            var imageData = await blobStorage.DownloadAsync(page.Path);

            var response = await llm.GenerateWithVisionAsync(
                """
                Analyze this comic book page and identify all characters visible.
                For each character, provide:
                1. Name (if recognizable, otherwise "Unknown")
                2. Bounding box coordinates [x, y, width, height] as percentages
                3. Confidence level (0-1)

                Return as JSON array.
                """,
                [imageData]);

            var detections = JsonSerializer.Deserialize<List<HeroDetection>>(response);
            if (detections != null)
            {
                foreach (var d in detections)
                {
                    d.PageNumber = page.PageNumber;
                    allDetections.Add(d);
                }
            }
        }

        return allDetections;
    }
}
```

### 4. Fan-Out/Fan-In Pattern

For processing many pages in parallel:

```csharp
[Function(nameof(ProcessPagesOrchestrator))]
public async Task<List<PageResult>> Run(
    [OrchestrationTrigger] TaskOrchestrationContext context)
{
    var input = context.GetInput<ProcessPagesInput>()!;

    // Fan-out: Start all page processing tasks
    var tasks = input.Pages.Select(page =>
        context.CallActivityAsync<PageResult>(
            nameof(ProcessSinglePageActivity),
            new SinglePageInput(input.ComicId, page)));

    // Fan-in: Wait for all to complete
    var results = await Task.WhenAll(tasks);

    return results.ToList();
}
```

### 5. Retry Policies

```csharp
[Function(nameof(FetchMetadataActivity))]
public async Task<ComicMetadata> Run(
    [ActivityTrigger] ComicProcessingInput input)
{
    // Activity will be retried on failure
}

// In orchestrator, specify retry options
var options = new TaskOptions(
    new TaskRetryOptions(
        firstRetryInterval: TimeSpan.FromSeconds(5),
        maxNumberOfAttempts: 3,
        backoffCoefficient: 2.0));

var metadata = await context.CallActivityAsync<ComicMetadata>(
    nameof(FetchMetadataActivity),
    input,
    options);
```

### 6. Sub-Orchestrations

```csharp
[Function(nameof(MainOrchestrator))]
public async Task Run([OrchestrationTrigger] TaskOrchestrationContext context)
{
    var input = context.GetInput<MainInput>()!;

    // Call sub-orchestrator for complex operations
    var heroResults = await context.CallSubOrchestrationAsync<HeroResults>(
        nameof(HeroProcessingOrchestrator),
        new HeroInput(input.ComicId, input.Pages));

    // Continue with main orchestration
    await context.CallActivityAsync(
        nameof(FinalizeActivity),
        new FinalizeInput(input.ComicId, heroResults));
}
```

## Job Status Notifications

```csharp
// Infrastructure/Services/JobStatusNotificationService.cs
public class JobStatusNotificationService(
    IRedisConnectionMultiplexer redis) : IJobStatusNotificationService
{
    public async Task NotifyStatusChangeAsync(Guid jobId, string status, string? details = null)
    {
        var db = redis.GetDatabase();
        var message = JsonSerializer.Serialize(new JobStatusMessage
        {
            JobId = jobId,
            Status = status,
            Details = details,
            Timestamp = DateTime.UtcNow
        });

        await db.PublishAsync(new RedisChannel($"job:{jobId}", RedisChannel.PatternMode.Literal), message);
    }
}

// API Controller for SSE streaming
[HttpGet("{id:guid}/stream")]
public async Task StreamStatus(Guid id, CancellationToken ct)
{
    Response.ContentType = "text/event-stream";

    await foreach (var status in _jobService.StreamStatusAsync(id, ct))
    {
        await Response.WriteAsync($"data: {JsonSerializer.Serialize(status)}\n\n", ct);
        await Response.Body.FlushAsync(ct);
    }
}
```

## Configuration

```json
// host.json
{
  "version": "2.0",
  "extensions": {
    "durableTask": {
      "storageProvider": {
        "type": "AzureStorage"
      },
      "maxConcurrentActivityFunctions": 10,
      "maxConcurrentOrchestratorFunctions": 5
    }
  },
  "logging": {
    "applicationInsights": {
      "samplingSettings": {
        "isEnabled": true
      }
    }
  }
}
```

## Models

```csharp
// Models/ComicProcessingInput.cs
public record ComicProcessingInput
{
    public required string BlobName { get; init; }
    public required Guid ComicId { get; init; }
    public required Guid UserId { get; init; }
}

public record ProcessingResult(
    bool Success,
    Guid ComicId,
    int PageCount = 0,
    int HeroCount = 0,
    string? Error = null
);

public record PageInfo(int PageNumber, string Path);

public record HeroDetection
{
    public string Name { get; set; } = "";
    public int PageNumber { get; set; }
    public BoundingBox BoundingBox { get; set; } = new();
    public float Confidence { get; set; }
}

public record BoundingBox(float X, float Y, float Width, float Height);
```

## Local Development

```bash
# Start Azurite storage emulator
azurite --silent --location ./azurite --blobHost 0.0.0.0

# Run functions locally
cd src/ComicRag.Functions
func start

# Or via Aspire (recommended)
dotnet run --project src/ComicRag.AppHost
```

## Anti-Patterns

```csharp
// NEVER do I/O directly in orchestrator
[Function(nameof(BadOrchestrator))]
public async Task Run([OrchestrationTrigger] TaskOrchestrationContext context)
{
    // WRONG - I/O in orchestrator
    var data = await _httpClient.GetAsync("https://api.example.com");

    // RIGHT - Call activity for I/O
    var data = await context.CallActivityAsync<Data>(
        nameof(FetchDataActivity), input);
}

// NEVER use non-deterministic code in orchestrator
var random = new Random().Next();  // WRONG
var now = DateTime.UtcNow;         // WRONG
var guid = Guid.NewGuid();         // WRONG

// RIGHT - Use context for deterministic values
var now = context.CurrentUtcDateTime;
```
