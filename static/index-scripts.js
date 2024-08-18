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
    embededThumbnailContainer.style.display = 'none';
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
    const resultsContainer = document.getElementById('results-text');

    // Hide buttons initially.
    console.log("Hiding upload, extract, and embed buttons.");
    uploadButton.style.display = 'none';
    extractButton.style.display = 'none';
    embedButton.style.display = 'none';
    console.log("Clearing thumbnails and processing results.");
    clearThumbnails();
    clearProcessingResults();

    // Create a container for the thumbnail if not already present.
    let thumbnailContainer = document.getElementById('thumbnailContainer');
    if (!thumbnailContainer) {
        thumbnailContainer = document.createElement('div');
        thumbnailContainer.id = 'thumbnailContainer';
        thumbnailContainer.classList.add('thumbnail-container');
        resultsContainer.appendChild(thumbnailContainer);
        console.log("Created new thumbnail container and appended to results container.");
    } else {
        console.log("Using existing thumbnail container.");
    }

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
        } else {
            alert('Please select a PNG image.');
            console.log("Hiding upload button for invalid browsed image.");
            uploadButton.style.display = 'none';
        }
    } else {
        fileLabel.textContent = 'No file selected';
        console.log("Hiding upload button as no browsed image.");
        uploadButton.style.display = 'none';
    }
});

// Event listener for Upload button, and processing.
document.getElementById('uploadButton').addEventListener('click', function() {
    console.log("Request to upload the browsed image.");
    const fileInput = document.getElementById('imageUpload');
    const file = fileInput.files[0];
    const uploadButton = document.getElementById('uploadButton');
    const extractButton = document.getElementById('extractButton');
    const embedButton = document.getElementById('embedButton');
    const formData = new FormData();

    formData.append('file', file);

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
        return response.json();
    })
    .then(data => {
        console.log("Hiding upload button as already uploaded.");
        uploadButton.style.display = 'none';
        const resultsElement = document.getElementById('processingResults');
        console.log("Displaying results from /upload endpoint.");
        resultsElement.textContent = `File coded: ${data.coded}, 
                                        Password protected: ${data.password},
                                        Embedding capacity: ${data.capacity} bytes`;
        
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

    // Initialize the list of files to embed.
    console.log("Initialising list of files to embed.");
    fileEmbedList.innerHTML = '';
    fileEmbedList.filesArray = [];

    // Display files to embed section.
    console.log("Displaying files to embed section.");
    embedSection.style.display = 'block';
});

// Event listener for files to embed browser.
document.getElementById('fileEmbed').addEventListener('change', function(event) {
    console.log("Browsing for files to embed into uploaded image.");
    const files = event.target.files;
    const fileEmbedList = document.getElementById('fileEmbedList');
    const filesArray = fileEmbedList.filesArray || [];

    // Add new files to the existing list.
    for (let i = 0; i < files.length; i++) {
        filesArray.push(files[i]);

        const li = document.createElement('li');
        li.textContent = files[i].name;
        fileEmbedList.appendChild(li);
    }

    // Store the updated list of files in the fileEmbedList element.
    fileEmbedList.filesArray = filesArray;

    console.log("File list updated from user selection.", files);
});

// Event listener for commit files to embed.
document.querySelector('label[for="fileEmbed"]').addEventListener('click', function() {
    console.log("Committing files for embedding...");
    document.getElementById('fileEmbed').click();
});

// Event listener for embed files button.
document.getElementById('embedSubmitButton').addEventListener('click', function() {
    const embedFiles = document.getElementById('fileEmbed').files;
    if (embedFiles.length === 0) {
        console.log("No files selected to embed.");
        alert('Please select at least one file to embed.');
        return;
    }

    console.log("File list created for user selection: ", embedFiles);

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

    for (let i = 0; i < embedFiles.length; i++) {
        formData.append('files', embedFiles[i]);
        console.log("Appending file to form data: ", embedFiles[i]);
    }

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
            const embeddedImageThumbnail = document.getElementById('embeddedImageThumbnail');
            embeddedImageThumbnail.src = data.thumbnail;

            const embeddedFileName = document.getElementById('embeddedImageFileName');
            embeddedFileName.textContent = data.filename;
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
    if (requiresPassword) {
        // Display password modal dialog.
        const modal = document.getElementById('passwordModal');
        modal.style.display = 'block';

        // Focus on the password input field.
        const passwordInput = document.getElementById('submitPasswordInput');
        if (passwordInput) {
            passwordInput.focus();
        }   
    } else {
        console.log("Performing embedded file extraction.");
        performExtraction();
    }
});

// Event listener for extract password submit.
document.getElementById('extractPasswordSubmitButton').addEventListener('click', function() {
    const password = document.getElementById('submitPasswordInput').value;
    const modal = document.getElementById('passwordModal');
    modal.style.display = 'none';
    console.log("Performing embedded file extraction (with password).");
    performExtraction(password);
});

// Event listener for enter key to submit extraction password.
document.getElementById('submitPasswordInput').addEventListener('keypress', function(event) {
    if (event.key === 'Enter') {
        console.log("Enter key pressed to perform extraction.");
        event.preventDefault();
        document.getElementById('extractPasswordSubmitButton').click();
    }
});

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
    
        // Preserve the original image thumbnail by cloning it.
        const originalImageDiv = document.getElementById('originalThumbnail');
        const resultsTextDiv = document.getElementById('results-text');
    
        // Clear extracted file thumbnails only.
        resultsTextDiv.innerHTML = '';
    
        // Re-append the original image thumbnail if it exists (cloneNode ensures it's not detached).
        // Deep clone the original thumbnail and append.
        if (originalImageDiv) {
            const originalClone = originalImageDiv.cloneNode(true);
            resultsTextDiv.appendChild(originalClone);
        }
       
        const files = JSON.parse(data.files);
        files.forEach(file => {
            const fileDiv = document.createElement('div');
            fileDiv.classList.add('file-thumbnail');

            // File type of extracted file.
            console.log('Extracted file: ' + file.name);
            console.log('Extracted file of type: ' + file.type);

            if (file.type.startsWith('image/')) {
                // IMAGE mime types.
                // Show image as thumbnail.
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

                // Create and append a paragraph element with the file name.
                const fileName = document.createElement('p');
                fileName.textContent = file.name;
                fileName.classList.add('thumbnail-filename');
                fileDiv.appendChild(fileName);

            } else if (file.type.startsWith('text/')) {
                // TEXT mime types.
                // Show image as text thumbnail.
                const a = document.createElement('a');
                a.href = file.path;
                a.download = file.name; // Enable download
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
            } else {
                const a = document.createElement('a');
                a.href = file.path;
                a.target = '_blank';
                const img = document.createElement('img');
                img.src = '/static/icon-generic.png';
                img.classList.add('thumbnail');
                a.appendChild(img);
                fileDiv.appendChild(a);

                const fileName = document.createElement('p');
                fileName.textContent = file.name;
                fileName.classList.add('thumbnail-filename');
                fileDiv.appendChild(fileName);
            }

            resultsTextDiv.appendChild(fileDiv);
        });

        if (data.extracted === "True") {
            resultsElement.className = 'results-text coded';
            extractButton.style.display = 'none';
        } else {
            resultsElement.className = 'results-text not-coded';
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

// Function to toggle border on an image.
function toggleBorder(imgElement, borderOn) {
    if (borderOn) {
        console.log('Setting border ON for image.');
        imgElement.classList.add('border-on');
    } else {
        console.log('Setting border OFF for image.');
        imgElement.classList.remove('border-on');
    }
}
