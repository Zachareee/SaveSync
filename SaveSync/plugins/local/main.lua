File_included = false

function Info()
	return {
		name = "Test",
		description = "Local plugin for testing",
		author = "Zachareee",
	}
end

function Init(credentials)
	return "Credentials"
end

FILEMAPS = {
	FAKE = "fake_folder",
}

--- Sync function
--- @param tag string Tagname for the folder
--- @param filename string Filename to upload
--- @param date_modified number Last modified date
--- @param zipbuffer string Buffer of the zipfile created
function Upload(tag, filename, date_modified, zipbuffer)
	print(date_modified)
	local file = io.open(FILEMAPS[tag] .. "/" .. filename .. ".zip", "wb")
	print(file)
	if file then
		file:write(zipbuffer)
		file:close()
	end
end

function Remove(tag, filename)
	print(os.remove(FILEMAPS[tag] .. "/" .. filename .. ".zip"))
end

---comment
---@return {tag: string, filename: string, last_modified: {secs_since_epoch: number, nanos_since_epoch: string}, data: string?}[]?
function Read_cloud()
	local content = nil
	if File_included then
		local file = io.open("fake_folder/fake.zip", "rb")
		if not file then
			return nil
		end

		content = file:read("*a")
		file:close()
	end

	return {
		{
			tag = "FAKE",
			folder_name = "fake",
			last_modified = { secs_since_epoch = os.time(), nanos_since_epoch = 0 },
			-- last_modified = { secs_since_epoch = 0, nanos_since_epoch = 0 },
			data = content,
		},
	}
end

function Download(tag, filename)
	local file = io.open("fake_folder/fake.zip", "rb")
	print(file)
	if not file then
		return nil
	end

	local content = file:read("*a")
	file:close()
	print("Content is", content)
	return content
end
