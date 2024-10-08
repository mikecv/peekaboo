// Function to display wait spinner.
function showSpinner() {
    console.log("Showing spinner...");
    document.getElementById('overlay').classList.add('active');
}

// Function to hide wait spinner.
function hideSpinner() {
    console.log("Hiding spinner...");
    document.getElementById('overlay').classList.remove('active');
}

// Initialise value of image file browsed.
document.addEventListener('DOMContentLoaded', function() {
    console.log("Listening for image browsing.");
    document.getElementById('imageUpload').value = '';
});

// Function to clear the thumbnails previously displayed.
function clearThumbnails() {
    console.log("Clearing any result thumbnails.");
    const resultsTextDiv = document.getElementById('results-text');
    const embededThumbnailContainer = document.getElementById('embeddedImageContainer');
    const extractThumbnailContainer = document.getElementById('extractedResultsContainer');
    embededThumbnailContainer.style.display = 'none';
    extractThumbnailContainer.style.display = 'none';
    resultsTextDiv.innerHTML = '';
}

let requiresPassword = false;

// Function to clear the processing results.
function clearProcessingResults() {
    const resultsElement = document.getElementById('processingResults');
    resultsElement.textContent = '';
    resultsElement.className = 'results-text';
}

// Function to handle image upload.
document.getElementById('imageUpload').addEventListener('change', function(event) {
    console.log("Request to upload the browsed image.");

    // Initialize elements
    const file = event.target.files[0];
    const fileLabel = document.getElementById('fileLabel');
    const uploadButton = document.getElementById('uploadButton');
    const extractButton = document.getElementById('extractButton');
    const embedButton = document.getElementById('embedButton');

    // Hide buttons initially.
    console.log("Hiding upload, extract, and embed buttons.");
    uploadButton.style.display = 'none';
    extractButton.style.display = 'none';
    embedButton.style.display = 'none';
    console.log("Clearing thumbnails and processing results.");
    clearThumbnails();
    clearProcessingResults();

    // Reset partial workflows if they exist.
    // This allows aborting a workflow and browsing of a new inage.
    console.log("Image upload input changed - triggering resetWorkflow.");
    resetWorkflow();

    // Handle embeded file type selection.
    if (file) {
        fileLabel.textContent = file.name;
        if (file.type === 'image/png') {
            const reader = new FileReader();
            reader.onload = function(e) {
                let imgLink = document.getElementById('thumbnailLink');
                let img = document.getElementById('thumbnail');
                if (!imgLink) {
                    imgLink = document.createElement('a');
                    imgLink.id = 'thumbnailLink';
                    imgLink.target = '_blank';
                    thumbnailContainer.appendChild(imgLink);
                }
                if (!img) {
                    img = document.createElement('img');
                    img.id = 'thumbnail';
                    img.classList.add('thumbnail');
                    imgLink.appendChild(img);
                } else if (img.parentElement !== imgLink) {
                    // Ensure the img is inside the imgLink.
                    imgLink.appendChild(img);
                }
                img.src = e.target.result;
                imgLink.href = e.target.result;
                img.style.display = 'block';

                console.log("Displaying upload button for valid browsed image.");
                uploadButton.style.display = 'block';
            }
            reader.readAsDataURL(file);

            uploadButton.style.display = 'block';
            fileLabel.textContent = file.name;
            fileLabel.style.display = 'inline';
            thumbnailContainer.style.display = 'block';
        } else {
            alert('Please select a PNG image.');
            console.log("Hiding upload button for invalid browsed image.");
            uploadButton.style.display = 'none';
            fileLabel.style.display = 'none';
            thumbnailContainer.style.display = 'none';
        }
    } else {
        fileLabel.textContent = 'No file selected';
        console.log("Hiding upload button as no browsed image.");
        uploadButton.style.display = 'none';
    }
});

// Global variable to hold embedding capacity.
// This will start out as the embedding capacity returned by the /upload endpoint,
// and then be decremented as files are selected for embedding.
let startingCapacity = 0;
let embeddingCapacity = 0;
let embedCapacityPercent = 0;
// Also define warning and high embedding limits.
let capacityMedium = 0;
let capacityHigh = 0;
// Also initialise parameter which is decremented for each file.
let overheadPerFile = 0;

// Event listener for Upload button, and processing.
document.getElementById('uploadButton').addEventListener('click', function() {
    console.log("Request to upload the browsed image.");
    const fileInput = document.getElementById('imageUpload');
    const file = fileInput.files[0];
    const uploadButton = document.getElementById('uploadButton');
    const extractButton = document.getElementById('extractButton');
    const embedButton = document.getElementById('embedButton');
    const formData = new FormData();

    // Initialise extract files password entry attempts.
    passwordAttempts = 0;

    formData.append('file', file);

    // Show the progress spinner.
    showSpinner();

    console.log("Posting to /upload endpoint.");
    fetch('/upload', {
        method: 'POST',
        body: formData
    })
    .then(response => {
        if (!response.ok) {
            throw new Error('Failed to upload file.');
        }

        console.log("Upload of browsed file successful.");
        // Hide the progress spinner.
        hideSpinner();

        return response.json();
        })
    .then(data => {
        console.log("Hiding upload button as already uploaded.");
        uploadButton.style.display = 'none';
        const resultsElement = document.getElementById('processingResults');
        console.log("Displaying results from /upload endpoint.");

        // Store the initial embedding capacity.
        startingCapacity = parseInt(data.capacity, 10);
        embeddingCapacity = parseInt(data.capacity, 10);
        overheadPerFile = parseInt(data.overhead, 10);
        console.log("Initial embedding capacity without overhead: " + startingCapacity);

        // Medium and high levels for warnings on amount of capacicty left.
        capacityMedium = parseInt(embeddingCapacity * 0.65, 10);
        capacityHigh = parseInt(embeddingCapacity * 0.5, 10);

        // Overhead per file to use when calculating remaining capacity.
        console.log("Overhead per embedded file: " + overheadPerFile);

        // Need to add overhead for 1 file as a buffer.
        embeddingCapacity -= overheadPerFile;
        console.log("Initial embedding capacity: " + embeddingCapacity);

        // Embedding capacity will start with the returned value.
        // It will be decremented as files are selected for embedding.
        resultsElement.textContent = `File coded: ${data.coded}, 
                                        Password protected: ${data.password},
                                        Embed capacity: ${embeddingCapacity} bytes`;
 
        requiresPassword = data.password === "True";

        if (data.coded === "True") {
            resultsElement.className = 'results-text coded';
            extractButton.style.display = 'block';
            embedButton.style.display = 'block';
        } else {
            resultsElement.className = 'results-text not-coded';
            embedButton.style.display = 'block';
        }
    })
    .catch(error => {
        console.error('Error:', error);
    });
});

// Event listener for Embed images button.
document.getElementById('embedButton').addEventListener('click', function() {
    console.log("Request to embed files into uploaded image.");
    const embedSection = document.getElementById('embedSection');
    const fileEmbedList = document.getElementById('fileEmbedList');

    // Don't need the embed or extract buttons any more, so hide them.
    // This reduces the reset path to just the Browse for new image button.
    console.log("Hiding Embed and Extract buttons as not needed.");
    extractButton.style.display = 'none';
    embedButton.style.display = 'none';

    // Initialize the list of files to embed.
    console.log("Initialising list of files to embed.");
    fileEmbedList.innerHTML = '';
    fileEmbedList.filesArray = [];

    // Display files to embed section.
    console.log("Displaying files to embed section.");
    embedSection.style.display = 'block';
});

// Global variable to store the valid files to embed list.
// That is, files that don't exceed the embedding capacity.
let validFiles = [];

// Function to update the embedding capacity display.
// Only add files that fit within the embedded capacity limit.
// Function to update the embedding capacity display
function updateEmbeddingCapacityDisplay() {
    const resultsElement = document.getElementById('processingResults');

    // Put border around capacity results, with colour according to criticality.
    if (resultsElement) {
        if (embeddingCapacity < capacityHigh) {
            resultsElement.className = 'results-text high';
            // resultsElement.textContent = `Embedding capacity remaining: ${embeddingCapacity} bytes ${embedCapacityPercent} %)`;
        }
        else if (embeddingCapacity < capacityMedium) {
            resultsElement.className = 'results-text medium';
            // resultsElement.textContent = `Embedding capacity remaining: ${embeddingCapacity} bytes remaining.`;
        }
        else {
            resultsElement.className = 'results-text low';
            // resultsElement.textContent = `Embedding capacity remaining: ${embeddingCapacity} bytes remaining.`;
        }
        resultsElement.textContent = `Embedding capacity: ${embeddingCapacity} bytes (${embedCapacityPercent.toFixed(1)} %)`;
    }
}

// Event listener for files to embed browser.
document.getElementById('fileEmbed').addEventListener('change', function(event) {
    console.log("Browsing for files to embed into uploaded image.");
    const files = Array.from(event.target.files);
    const fileEmbedList = document.getElementById('fileEmbedList');
    const filesArray = fileEmbedList.filesArray || [];
    let totalFileSize = 0;

    // Calculate total file size for the current files selection.
    // Include the overhead per file required when embendding files.
    files.forEach(file => {
        totalFileSize += file.size + overheadPerFile;
    });

    // Check if adding these files would exceed embedding capacity.
    // If so, keep any previous selected files, but not the new selection.
    if (totalFileSize > embeddingCapacity) {
        // If it exceeds, show an alert and don't include the last file(s).
        alert("You have exceeded the embedding capacity limit for this image. Please select smaller files.");
        
        // Only add files that fit within the capacity.
        const fittingFiles = files.filter(file => file.size <= embeddingCapacity);

        // Add the valid fitting files to the validFiles array.
        validFiles.push(...fittingFiles);
        
        // Update embedding capacity by subtracting valid files' sizes.
        fittingFiles.forEach(file => {
            embeddingCapacity -= file.size + overheadPerFile;
        });

        // Calculate the percentage of original original capacity used.
        embedCapacityPercent =  ((startingCapacity - embeddingCapacity) / startingCapacity) * 100;
        console.log("Capacity used (%): " + embedCapacityPercent.toFixed(1));
    } else {
        // If all files fit, add them to validFiles.
        validFiles.push(...files);

        // Decrement embedding capacity for each valid file
        // Include the overhead per file required when embendding files.
        files.forEach(file => {
            embeddingCapacity -= file.size + overheadPerFile;
        });

        // Calculate the percentage of original original capacity used.
        embedCapacityPercent =  (embeddingCapacity / startingCapacity) * 100;
        console.log("Capacity used (%): " + embedCapacityPercent.toFixed(1));
    }

    // Update the embedding capacity display.
    updateEmbeddingCapacityDisplay();

    // Clear the file input to allow for new selections.
    event.target.value = '';

    // Display the list of valid selected files.
    displaySelectedFiles();
});

// Function to display the valid selected files.
function displaySelectedFiles() {
    const fileEmbedList = document.getElementById('fileEmbedList');
     // Clear previous list.
    fileEmbedList.innerHTML = '';
    validFiles.forEach(file => {
        const li = document.createElement('li');
        li.textContent = `${file.name} (${file.size} bytes)`;
        fileEmbedList.appendChild(li);
    });
}

// Event listener for commit files to embed.
document.querySelector('label[for="fileEmbed"]').addEventListener('click', function() {
    console.log("Committing files for embedding...");
    document.getElementById('fileEmbed').click();
});

// Event listener for embed files button.
document.getElementById('embedSubmitButton').addEventListener('click', function() {
    const embedFiles = document.getElementById('fileEmbed').files;

    // Check if no files to submit, if so warn and prevent submit.
    if (validFiles.length === 0) {
        console.log("No files selected to embed.");
        alert('Please select at least one file to embed.');
        return;
    }

    // Valid file list for submission.
    console.log("Files selected for embedding: ", validFiles);

    // Display a modal dialog to enter a password (blank if not required).
    console.log("Getting embed password.");
    const modal = document.getElementById('embedPasswordModal');
    modal.style.display = 'block';

    // Focus on the password input field.
    const passwordInput = document.getElementById('embedPasswordInput');
    if (passwordInput) {
        passwordInput.focus();
    }   
});

// Event listener for submiting embed password.
document.getElementById('embedPasswordSubmitButton').addEventListener('click', function() {
    console.log("Embed files submit button pressed.");
    const password = document.getElementById('embedPasswordInput').value;
    const modal = document.getElementById('embedPasswordModal');
    modal.style.display = 'none';

    console.log("Calling function to submit files for embedding.");
    performEmbedding(password);
});

// Event listener for submiting embed password.
// Submitting with enter key instead of submit button.
document.getElementById('embedPasswordInput').addEventListener('keypress', function(event) {
    if (event.key === 'Enter') {
        console.log("Enter key pressed to submit files for embedding.");
        event.preventDefault();
        document.getElementById('embedPasswordSubmitButton').click();
    }
});

// Embedding password worker function.
function performEmbedding(password = '') {
    const fileEmbedList = document.getElementById('fileEmbedList');
    const embedFiles = fileEmbedList.filesArray || [];
    const formData = new FormData();

    console.log("Inside function to post to embedding endpoint.");

    // Add valid files to FormData
    validFiles.forEach(file => {
        formData.append('files', file);
    });
    
    formData.append('password', password);

    // Show the progress spinner.
    showSpinner();

    console.log("Posting to /embed endpoint.");
    fetch('/embed', {
        method: 'POST',
        body: formData
    })
    .then(response => {
        if (!response.ok) {
            throw new Error('Failed to embed data.');
        }
        // Hide the progress spinner.
        hideSpinner();

        return response.json();
    })
    .then(data => {
        console.log("Received data from /embed endpoint.");
        const resultsElement = document.getElementById('processingResults');
        resultsElement.textContent = `File(s) embedded: ${data.embedded}, Duration: ${data.time}`;

        if (data.thumbnail) {
            console.log("Displaying thumbnail of image after embedding.");

            // Clear only the embedding results container, not the original thumbnail.
            const thumbnailContainer = document.getElementById('embeddedImageContainer');
            thumbnailContainer.style.display = 'block';
    
            // Instead of clearing the entire container, just remove the old thumbnail if necessary.
            const oldThumbnail = document.getElementById('embeddedImageThumbnail');
            if (oldThumbnail) {
                oldThumbnail.remove();
            }
    
            // Create the anchor element for opening in a new tab.
            const imgLink = document.createElement('a');
            imgLink.href = data.thumbnail;
            imgLink.target = '_blank';

            // Create the image element for the embedded image thumbnail.
            const embeddedImageThumbnail = document.createElement('img');
            embeddedImageThumbnail.id = 'embeddedImageThumbnail';
            embeddedImageThumbnail.classList.add('thumbnail');
            embeddedImageThumbnail.src = data.thumbnail;
            embeddedImageThumbnail.alt = 'Embedded image thumbnail';

            // Append the image inside the anchor, and the anchor inside the container
            imgLink.appendChild(embeddedImageThumbnail);
            thumbnailContainer.appendChild(imgLink);

            // Check if the element for the file name exists before updating it.
            const embeddedFileName = document.getElementById('embeddedImageFileName');
            if (embeddedFileName) {
                embeddedFileName.textContent = data.filename;
            } else {
                console.error("The embedded file name element is missing from the DOM.");
            }
        } else {
            console.error('Thumbnail URL is missing or invalid');
        }

        // Clear the embed section.
        document.getElementById('fileEmbedList').innerHTML = '';
        document.getElementById('embedSection').style.display = 'none';
    })
    .catch(error => {
        console.error('Error:', error);
    });
}

document.querySelectorAll('.close').forEach(closeButton => {
    closeButton.addEventListener('click', function() {
        console.log("Query selector clicked");
        const modal = this.closest('.modal');
        modal.style.display = 'none';
    });
});

// Event listener for extract button selected.
document.getElementById('extractButton').addEventListener('click', function() {

    // Don't need the embed or extract buttons any more, so hide them.
    // This reduces the reset path to just the Browse for new image button.
    console.log("Hiding Embed and Extract buttons as not needed.");
    extractButton.style.display = 'none';
    embedButton.style.display = 'none';

    if (requiresPassword) {
        // Display password modal dialog.
        const modal = document.getElementById('extractPasswordModal');
        modal.style.display = 'block';

        // Focus on the password input field.
        const passwordInput = document.getElementById('extractPasswordInput');
        if (passwordInput) {
            passwordInput.focus();
        }   
    } else {
        console.log("Performing embedded file extraction.");
        performExtraction();
    }
});

// Function to manually trigger the event listener for the Extract button.
// This is used to retry extraction if passwork entered wrong.
function extract_listener() {
    document.getElementById('extractButton').click();
}

// Event listener for extract password submit.
document.getElementById('extractPasswordSubmitButton').addEventListener('click', function() {
    const password = document.getElementById('extractPasswordInput').value;
    const modal = document.getElementById('extractPasswordModal');
    modal.style.display = 'none';
    console.log("Performing embedded file extraction (with password).");
    performExtraction(password);
});

// Event listener for enter key to submit extraction password.
document.getElementById('extractPasswordInput').addEventListener('keypress', function(event) {
    if (event.key === 'Enter') {
        console.log("Enter key pressed to perform extraction.");
        event.preventDefault();
        document.getElementById('extractPasswordSubmitButton').click();
    }
});

// Global variable to hold password attempts.
let passwordAttempts= 0;

// Worker function to perform extraction.
function performExtraction(password = '') {
    // Show the progress spinner.
    showSpinner();

    const formData = new FormData();
    formData.append('password', password);

    console.log("Posting to /extract endpoint.");
    fetch('/extract', {
        method: 'POST',
        body: new URLSearchParams(formData)
    })
    .then(response => {
        if (!response.ok) {
            throw new Error('Failed to extract data.');
        }

        // Hide the progress spinner.
        hideSpinner();
        return response.json();
    })
    .then(data => {
        console.log("Received data from /extract endpoint.");
        const resultsElement = document.getElementById('processingResults');
        resultsElement.textContent = `File(s) extracted: ${data.extracted}, Duration: ${data.time}`;

        // Check for errors in extraction.
        // Check for wrong password entered.
        if (data.extracted === "Incorrect password provided") {
            resultsElement.textContent = `Error: ${data.extracted}, Duration: ${data.time}`;
            resultsElement.className = 'results-text error';
            extractButton.style.display = 'none';

            // Increment password attempts.
            passwordAttempts += 1;

            // Present an alert regarding the wrong password.
            // Only get 3 attempts.
            if (passwordAttempts < 4){
                alert('Wrong password entered, attempt (' + passwordAttempts + ' of 3)');
                console.log("Wrong password entered attenpting to extract files.");
            }
            // Trigger the extract listener manually.
            // Not part of 'if' above as still want user to acknowledge last attempt.
            if (passwordAttempts < 3){
                extract_listener();
            }
        } else {
            // Reinitialise password attempt counter.
            passwordAttempts = 0;

            // Show the thumbnails container
            const extractedResultsContainer = document.getElementById('extractedResultsContainer');
            extractedResultsContainer.style.display = 'block';
        
            const extractedFileResultsDiv = document.getElementById('extractedFileResults');
            extractedFileResultsDiv.innerHTML = '';
        
            // Preserve the original image thumbnail by cloning it.
            const originalImageDiv = document.getElementById('originalThumbnail');
            if (originalImageDiv) {
                const originalClone = originalImageDiv.cloneNode(true);
                extractedFileResultsDiv.appendChild(originalClone);
            }
                
            const files = JSON.parse(data.files);
            files.forEach(file => {
                const fileDiv = document.createElement('div');
                fileDiv.classList.add('file-thumbnail');

                // File type of extracted file.
                console.log('Extracted file: ' + file.name);
                console.log('Extracted file of type: ' + file.type);

                if (file.type.startsWith('image/')) {
                    //
                    // IMAGE mime types.
                    // Show image as actual image thumbnail.
                    //
                    const a = document.createElement('a');
                    a.href = file.path;
                    a.target = '_blank';
                    const img = document.createElement('img');
                    img.src = file.path;
                    img.alt = file.name;
                    img.classList.add('thumbnail');
                    img.classList.add('border-on');
                    a.appendChild(img);
                    fileDiv.appendChild(a);

                    // Apply the border to the image using the toggleBorder function.
                    // Only need to apply this check for image types as other types can't be encodded.
                    if (file.coded === true || file.coded === "true") {
                        // Border thumbnails for encoded images in green.
                        toggleBorder(img, true, 'green');
                    }
                    else {
                        // No border for non-coded thumbnails.
                        toggleBorder(img, false, 'grey');
                    }

                    // Create and append a paragraph element with the file name.
                    const fileName = document.createElement('p');
                    fileName.textContent = file.name;
                    fileName.classList.add('thumbnail-filename');
                    fileDiv.appendChild(fileName);
                }
                else if (file.type.startsWith('video/')) {
                    //
                    // VIDEO mime types.
                    // Show as video thumbnail.
                    //
                    const a = document.createElement('a');
                    a.href = file.path;
                    a.download = file.name;
                    const img = document.createElement('img');
                    img.src = '/static/icon-video.png';
                    img.alt = 'Video File Thumbnail';
                    img.classList.add('thumbnail');
                    a.appendChild(img);
                    fileDiv.appendChild(a);

                    const fileName = document.createElement('p');
                    fileName.textContent = file.name;
                    fileName.classList.add('thumbnail-filename');
                    fileDiv.appendChild(fileName);
                }
                else if (file.type.startsWith('text/')) {
                    //
                    // TEXT mime types.
                    // Show image as text thumbnail.
                    //
                    const a = document.createElement('a');
                    a.href = file.path;
                    a.download = file.name;
                    const img = document.createElement('img');
                    img.src = '/static/icon-text.png';
                    img.alt = 'Text File Thumbnail';
                    img.classList.add('thumbnail');
                    a.appendChild(img);
                    fileDiv.appendChild(a);

                    const fileName = document.createElement('p');
                    fileName.textContent = file.name;
                    fileName.classList.add('thumbnail-filename');
                    fileDiv.appendChild(fileName);
                }
                else if (file.type.startsWith('audio/')) {
                    //
                    // AUDIO mime types.
                    // Show as sound file thumbnail.
                    //
                    const a = document.createElement('a');
                    a.href = file.path;
                    a.download = file.name;
                    const img = document.createElement('img');
                    img.src = '/static/icon-audio.png';
                    img.alt = 'Audio File Thumbnail';
                    img.classList.add('thumbnail');
                    a.appendChild(img);
                    fileDiv.appendChild(a);

                    const fileName = document.createElement('p');
                    fileName.textContent = file.name;
                    fileName.classList.add('thumbnail-filename');
                    fileDiv.appendChild(fileName);
                }
                else if (file.type.startsWith('application/pdf')) {
                    //
                    // PDF mime type.
                    // Show as a PDF thumbnail.
                    //
                    const a = document.createElement('a');
                    a.href = file.path;
                    a.download = file.name;
                    const img = document.createElement('img');
                    img.src = '/static/icon-pdf.png';
                    img.alt = 'PDF File Thumbnail';
                    img.classList.add('thumbnail');
                    a.appendChild(img);
                    fileDiv.appendChild(a);

                    const fileName = document.createElement('p');
                    fileName.textContent = file.name;
                    fileName.classList.add('thumbnail-filename');
                    fileDiv.appendChild(fileName);
                }
                else if (file.type.startsWith('application/x-tar')) {
                    //
                    // Archive tar.gz mime type.
                    // Show tar.gz archive as archive thumbnail.
                    //
                    const a = document.createElement('a');
                    a.href = file.path;
                    a.download = file.name;
                    const img = document.createElement('img');
                    img.src = '/static/icon-archive.png';
                    img.alt = 'File Archive Thumbnail';
                    img.classList.add('thumbnail');
                    a.appendChild(img);
                    fileDiv.appendChild(a);

                    const fileName = document.createElement('p');
                    fileName.textContent = file.name;
                    fileName.classList.add('thumbnail-filename');
                    fileDiv.appendChild(fileName);
                }
                else {
                    //
                    // Not a specificly supported mime type.
                    // Show as a generic thumbnail.
                    //
                    const a = document.createElement('a');
                    a.href = file.path;
                    a.target = '_blank';
                    const img = document.createElement('img');
                    img.src = '/static/icon-generic.png';
                    img.alt = 'Generic File Thumbnail';
                    img.classList.add('thumbnail');
                    a.appendChild(img);
                    fileDiv.appendChild(a);

                    const fileName = document.createElement('p');
                    fileName.textContent = file.name;
                    fileName.classList.add('thumbnail-filename');
                    fileDiv.appendChild(fileName);
                }
                // Append the fileDiv to the thumbnails container
                extractedFileResultsDiv.appendChild(fileDiv);
            });

            if (data.extracted === "True") {
                resultsElement.className = 'results-text coded';
                extractButton.style.display = 'none';
            } else {
                resultsElement.className = 'results-text not-coded';
            }
        }
    })
    .catch(error => {
        console.error('Error:', error);
        // Hide the progress spinner.
        hideSpinner();
    });
}

// Close the modal when the user commits.
document.querySelector('.close').addEventListener('click', function() {
    const modal = document.getElementById('passwordModal');
    modal.style.display = 'none';
});

// Function to toggle border on an image thumbnail.
function toggleBorder(imgElement, borderOn, borderColor) {
    if (borderOn) {
        console.log('Setting border ON for image thumbnail.');
        imgElement.style.border = `2px solid ${borderColor}`;
        imgElement.style.padding = '5px';
    } else {
        console.log('Setting border OFF for image thumbnail.');
        imgElement.style.border = 'none';
    }
}

// Function for revealing passwoed entry locally.
function togglePasswordVisibility(inputId, iconId) {
    const passwordInput = document.getElementById(inputId);
    const eyeIcon = document.getElementById(iconId);
    
    if (passwordInput.type === "password") {
        passwordInput.type = "text";
        eyeIcon.classList.remove('fa-eye');
        eyeIcon.classList.add('fa-eye-slash');
    } else {
        passwordInput.type = "password";
        eyeIcon.classList.remove('fa-eye-slash');
        eyeIcon.classList.add('fa-eye');
    }
}

// Function to reset the workflow (clear all fields, thumbnails, and hide buttons).
function resetWorkflow() {
    console.log("Resetting workflow...");

    // Clear file selection for embedding.
    const fileEmbedInput = document.getElementById('fileEmbed');
    fileEmbedInput.value = '';
    const fileEmbedList = document.getElementById('fileEmbedList');
    fileEmbedList.innerHTML = '';
    fileEmbedList.filesArray = [];
    validFiles = [];

    // Clear any results or thumbnails.
    clearThumbnails();
    clearProcessingResults();

    // Hide all action buttons.
    document.getElementById('uploadButton').style.display = 'none';
    document.getElementById('embedButton').style.display = 'none';
    document.getElementById('extractButton').style.display = 'none';

    // Hide embed section and any previous embedded image.
    document.getElementById('embedSection').style.display = 'none';
    document.getElementById('embeddedImageContainer').style.display = 'none';
}
